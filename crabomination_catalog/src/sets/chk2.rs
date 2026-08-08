//! Champions of Kamigawa (CHK) gap wave: the Myojin cycle's divinity counters,
//! the slow-dual land cycle, the legends, and the Spirit/Arcane spells.
//! Tests in `classic_sets/chk`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, SpellSubtype,
    StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest,
};
use crate::mana::{Color, SpendRestriction, b, cost, g, generic, r, u, w};

fn types(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

fn arcane() -> Subtypes {
    Subtypes {
        spell_subtypes: vec![SpellSubtype::Arcane],
        ..Default::default()
    }
}

fn legendary() -> Vec<Supertype> {
    vec![Supertype::Legendary]
}

/// The CHK slow duals: "{T}: Add {C}" plus a coloured tap that keeps the land
/// down through its controller's next untap step.
fn slow_dual(name: &'static str, a: Color, c: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::OfColors(vec![a, c], Value::ONE),
                    },
                    Effect::SkipNextUntap { what: Selector::This },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

pub fn cloudcrest_lake() -> CardDefinition {
    slow_dual("Cloudcrest Lake", Color::White, Color::Blue)
}

pub fn lantern_lit_graveyard() -> CardDefinition {
    slow_dual("Lantern-Lit Graveyard", Color::Black, Color::Red)
}

pub fn pinecrest_ridge() -> CardDefinition {
    slow_dual("Pinecrest Ridge", Color::Red, Color::Green)
}

pub fn tranquil_garden() -> CardDefinition {
    slow_dual("Tranquil Garden", Color::Green, Color::White)
}

pub fn waterveil_cavern() -> CardDefinition {
    slow_dual("Waterveil Cavern", Color::Blue, Color::Black)
}

/// Hall of the Bandit Lord — legendary land, enters tapped. {T}, Pay 3 life:
/// Add {C}; a creature spell it funds gains haste.
pub fn hall_of_the_bandit_lord() -> CardDefinition {
    CardDefinition {
        name: "Hall of the Bandit Lord",
        card_types: vec![CardType::Land],
        supertypes: legendary(),
        static_abilities: vec![StaticAbility {
            description: "Hall of the Bandit Lord enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 3,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::ONE)),
                    SpendRestriction::CreatureHaste,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Untaidake, the Cloud Keeper — legendary land, enters tapped. {T}, Pay 2 life:
/// Add {C}{C}. Spend this mana only to cast legendary spells.
pub fn untaidake_the_cloud_keeper() -> CardDefinition {
    CardDefinition {
        name: "Untaidake, the Cloud Keeper",
        card_types: vec![CardType::Land],
        supertypes: legendary(),
        static_abilities: vec![StaticAbility {
            description: "Untaidake, the Cloud Keeper enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 2,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::Const(2))),
                    SpendRestriction::LegendarySpell,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Forbidden Orchard — {T}: Add one mana of any color. Whenever you tap it for
/// mana, target opponent creates a 1/1 colorless Spirit token.
pub fn forbidden_orchard() -> CardDefinition {
    CardDefinition {
        name: "Forbidden Orchard",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TappedForMana, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::Target(0),
                count: Value::ONE,
                definition: Box::new(spirit_token()),
            },
        }],
        ..Default::default()
    }
}

/// 1/1 colorless Spirit token.
fn spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Spirit]),
        ..Default::default()
    }
}

/// 1/1 green Snake token.
fn snake_token() -> TokenDefinition {
    TokenDefinition {
        name: "Snake".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: types(vec![CreatureType::Snake]),
        ..Default::default()
    }
}

// ── The Myojin cycle (CR 122.1 divinity counters) ────────────────────────────

/// The shared Myojin body: enters with a divinity counter if cast from hand,
/// indestructible while it has one, and one counter-removal activation.
fn myojin(
    name: &'static str,
    mana: crate::mana::ManaCost,
    pt: (i32, i32),
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Spirit]),
        power: pt.0,
        toughness: pt.1,
        enters_with_counters: Some((
            CounterType::Divinity,
            Value::IfPred {
                pred: Box::new(Predicate::CastFromHand),
                then: Box::new(Value::ONE),
                else_: Box::new(Value::ZERO),
            },
        )),
        static_abilities: vec![StaticAbility {
            description: "This creature has indestructible as long as it has a divinity counter.",
            effect: StaticEffect::SelfHasKeywordWhileCountersAtLeast {
                kind: CounterType::Divinity,
                n: 1,
                keyword: Keyword::Indestructible,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Divinity, 1)),
            effect,
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub fn myojin_of_cleansing_fire() -> CardDefinition {
    myojin(
        "Myojin of Cleansing Fire",
        cost(&[generic(5), w(), w(), w()]),
        (4, 6),
        Effect::Destroy {
            what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
        },
    )
}

pub fn myojin_of_infinite_rage() -> CardDefinition {
    myojin(
        "Myojin of Infinite Rage",
        cost(&[generic(7), r(), r(), r()]),
        (7, 4),
        Effect::Destroy {
            what: Selector::EachPermanent(R::Land),
        },
    )
}

pub fn myojin_of_seeing_winds() -> CardDefinition {
    myojin(
        "Myojin of Seeing Winds",
        cost(&[generic(7), u(), u(), u()]),
        (3, 3),
        Effect::Draw {
            who: Selector::You,
            amount: Value::count(Selector::EachPermanent(R::Permanent.and(R::ControlledByYou))),
        },
    )
}

pub fn myojin_of_lifes_web() -> CardDefinition {
    myojin(
        "Myojin of Life's Web",
        cost(&[generic(6), g(), g(), g()]),
        (8, 8),
        Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: R::Creature,
            count: Value::Const(20),
            tapped: false,
            haste: false,
            sacrifice_eot: false,
                return_eot: false,
                then: None,
        },
    )
}

// ── Legends ──────────────────────────────────────────────────────────────────

/// Azami, Lady of Scrolls — {2}{U}{U}{U} 0/2. Tap an untapped Wizard you
/// control: Draw a card.
pub fn azami_lady_of_scrolls() -> CardDefinition {
    CardDefinition {
        name: "Azami, Lady of Scrolls",
        cost: cost(&[generic(2), u(), u(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 0,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Creature.and(R::HasCreatureType(CreatureType::Wizard))),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ben-Ben, Akki Hermit — {2}{R}{R} 1/1. {T}: deals damage to target attacking
/// creature equal to the number of untapped Mountains you control.
pub fn ben_ben_akki_hermit() -> CardDefinition {
    CardDefinition {
        name: "Ben-Ben, Akki Hermit",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Goblin, CreatureType::Shaman]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking)),
                amount: Value::count(Selector::EachPermanent(
                    R::HasLandType(crate::card::LandType::Mountain)
                        .and(R::Untapped)
                        .and(R::ControlledByYou),
                )),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dosan the Falling Leaf — {1}{G}{G} 2/2. Players can cast spells only during
/// their own turns.
pub fn dosan_the_falling_leaf() -> CardDefinition {
    CardDefinition {
        name: "Dosan the Falling Leaf",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Human, CreatureType::Monk]),
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Players can cast spells only during their own turns.",
            effect: StaticEffect::PlayersCastOnlyOnOwnTurn,
        }],
        ..Default::default()
    }
}

/// Hisoka, Minamo Sensei — {2}{U}{U} 1/3. {2}{U}, Discard a card: counter target
/// spell if it has the same mana value as the discarded card.
pub fn hisoka_minamo_sensei() -> CardDefinition {
    CardDefinition {
        name: "Hisoka, Minamo Sensei",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::CounterSpell {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::IsSpellOnStack.and(R::ManaValueEqualsDiscardedThisEffect),
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Iname, Death Aspect — {4}{B}{B} 4/4. ETB: you may search your library for any
/// number of Spirit cards and put them into your graveyard.
pub fn iname_death_aspect() -> CardDefinition {
    CardDefinition {
        name: "Iname, Death Aspect",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Spirit]),
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::HasCreatureType(CreatureType::Spirit),
            to: ZoneDest::Graveyard,
            count: Value::Const(20),
        })],
        ..Default::default()
    }
}

/// Iname, Life Aspect — {4}{G}{G} 4/4. When it dies you may exile it to return
/// any number of target Spirit cards from your graveyard to your hand.
pub fn iname_life_aspect() -> CardDefinition {
    CardDefinition {
        name: "Iname, Life Aspect",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Spirit]),
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Exile Iname to return your Spirit cards".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Exile,
                    },
                    Effect::ReturnGraveyardCardsToHand {
                        filter: R::HasCreatureType(CreatureType::Spirit),
                        max: Value::Const(20),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Sachi, Daughter of Seshiro — {2}{G}{G} 1/3. Other Snakes get +0/+1; Shamans
/// you control have "{T}: Add {G}{G}".
pub fn sachi_daughter_of_seshiro() -> CardDefinition {
    CardDefinition {
        name: "Sachi, Daughter of Seshiro",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Snake, CreatureType::Shaman]),
        power: 1,
        toughness: 3,
        static_abilities: vec![
            StaticAbility {
                description: "Other Snake creatures you control get +0/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Snake)
                            .and(R::ControlledByYou)
                            .and(R::OtherThanSource),
                    ),
                    power: 0,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Shamans you control have \"{T}: Add {G}{G}\".",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Shaman).and(R::ControlledByYou),
                    ),
                    ability: ActivatedAbility {
                        tap_cost: true,
                        effect: Effect::AddMana {
                            who: PlayerRef::You,
                            pool: ManaPayload::OfColor(Color::Green, Value::Const(2)),
                        },
                        ..Default::default()
                    },
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Shisato, Whispering Hunter — {3}{G} 2/2. Upkeep: sacrifice a Snake. Combat
/// damage to a player makes them skip their next untap step.
pub fn shisato_whispering_hunter() -> CardDefinition {
    CardDefinition {
        name: "Shisato, Whispering Hunter",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Snake, CreatureType::Warrior]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::HasCreatureType(CreatureType::Snake),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::SelfSource,
                ),
                effect: Effect::SkipPlayerUntapStep {
                    player: PlayerRef::Target(0),
                },
            },
        ],
        ..Default::default()
    }
}

/// Takeno, Samurai General — {5}{W} 3/3 with bushido 2. Each other Samurai you
/// control gets +1/+1 for each point of bushido it has.
pub fn takeno_samurai_general() -> CardDefinition {
    CardDefinition {
        name: "Takeno, Samurai General",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Human, CreatureType::Samurai]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Bushido(2)],
        static_abilities: vec![StaticAbility {
            description: "Each other Samurai you control gets +1/+1 for each point of bushido.",
            effect: StaticEffect::PumpPerBushido {
                filter: R::HasCreatureType(CreatureType::Samurai)
                    .and(R::ControlledByYou)
                    .and(R::OtherThanSource),
            },
        }],
        ..Default::default()
    }
}

/// The Unspeakable — {6}{U}{U}{U} 6/7 flying trample. Combat damage to a player
/// returns a target Arcane card from your graveyard to your hand.
pub fn the_unspeakable() -> CardDefinition {
    CardDefinition {
        name: "The Unspeakable",
        cost: cost(&[generic(6), u(), u(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Spirit]),
        power: 6,
        toughness: 7,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Return an Arcane card from your graveyard".into(),
                body: Box::new(Effect::ReturnGraveyardCardsToHand {
                    filter: R::HasSpellSubtype(SpellSubtype::Arcane),
                    max: Value::ONE,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Uyo, Silent Prophet — {4}{U}{U} 4/4 flying. {2}, Return two lands you control
/// to their owner's hand: Copy target instant or sorcery spell.
pub fn uyo_silent_prophet() -> CardDefinition {
    CardDefinition {
        name: "Uyo, Silent Prophet",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Moonfolk, CreatureType::Wizard]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            bounce_other_filter: Some((R::Land.and(R::ControlledByYou), 2)),
            effect: Effect::CopySpellMayChooseTargets {
                what: target_filtered(
                    R::IsSpellOnStack.and(
                        R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    ),
                ),
                count: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Godo, Bandit Warlord — {5}{R} 3/3. ETB: search for an Equipment. First attack
/// each turn untaps it and your Samurai and adds a combat phase.
pub fn godo_bandit_warlord() -> CardDefinition {
    CardDefinition {
        name: "Godo, Bandit Warlord",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Human, CreatureType::Barbarian]),
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Search your library for an Equipment card".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
            }),
            TriggeredAbility {
                event: EventSpec {
                    once_per_turn: true,
                    ..EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                },
                effect: Effect::Seq(vec![
                    Effect::Untap {
                        what: Selector::EachPermanent(
                            R::HasCreatureType(CreatureType::Samurai).and(R::ControlledByYou),
                        ),
                        up_to: None,
                    },
                    Effect::Untap {
                        what: Selector::This,
                        up_to: None,
                    },
                    Effect::AdditionalCombatPhase { count: Value::ONE },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Sensei Golden-Tail — {1}{W} 2/1 with bushido 1. {1}{W}, {T}: put a training
/// counter on target creature; it gains bushido 1 and becomes a Samurai.
pub fn sensei_golden_tail() -> CardDefinition {
    CardDefinition {
        name: "Sensei Golden-Tail",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Fox, CreatureType::Samurai]),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Bushido(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::Training,
                    amount: Value::ONE,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Bushido(1),
                    duration: Duration::Permanent,
                },
                Effect::AddCreatureTypes {
                    what: Selector::Target(0),
                    creature_types: vec![CreatureType::Samurai],
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Shimatsu the Bloodcloaked — {3}{R} 0/0. As it enters, sacrifice any number of
/// permanents; it enters with that many +1/+1 counters.
pub fn shimatsu_the_bloodcloaked() -> CardDefinition {
    CardDefinition {
        name: "Shimatsu the Bloodcloaked",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Demon, CreatureType::Spirit]),
        power: 0,
        toughness: 0,
        as_enters_effect: Some(Effect::SacrificeAnyNumber {
            who: PlayerRef::You,
            filter: R::Permanent,
            per_each: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        }),
        ..Default::default()
    }
}

/// Hikari, Twilight Guardian — {3}{W}{W} 4/4 flying. Casting a Spirit or Arcane
/// spell may blink it until the next end step.
pub fn hikari_twilight_guardian() -> CardDefinition {
    CardDefinition {
        name: "Hikari, Twilight Guardian",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Spirit]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Spirit)
                        .or(R::HasSpellSubtype(SpellSubtype::Arcane)),
                },
            ),
            effect: Effect::MayDo {
                description: "Exile Hikari until the next end step".into(),
                body: Box::new(Effect::ExileReturnToOwnerNextEndStep {
                    what: Selector::This,
                    tapped: false,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Auras ────────────────────────────────────────────────────────────────────

fn aura() -> Subtypes {
    Subtypes {
        enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
        ..Default::default()
    }
}

/// Aura of Dominion — {U}{U} Aura. {1}, Tap an untapped creature you control:
/// Untap enchanted creature.
pub fn aura_of_dominion() -> CardDefinition {
    CardDefinition {
        name: "Aura of Dominion",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_other_filter: Some(R::Creature.and(R::ControlledByYou)),
            effect: Effect::Untap {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                up_to: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Field of Reality — {2}{U} Aura. Enchanted creature can't be blocked by
/// Spirits. {1}{U}: Return this Aura to its owner's hand.
pub fn field_of_reality() -> CardDefinition {
    CardDefinition {
        name: "Field of Reality",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::HasCreatureType(
                CreatureType::Spirit,
            )))],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::ReturnSelf,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Midnight Covenant — {1}{B} Aura. Enchanted creature has "{B}: This creature
/// gets +1/+1 until end of turn."
pub fn midnight_covenant() -> CardDefinition {
    CardDefinition {
        name: "Midnight Covenant",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Oni Possession — {2}{B} Aura. Enchanted creature gets +3/+3, has trample and
/// is a Demon Spirit; at your upkeep you sacrifice a creature.
pub fn oni_possession() -> CardDefinition {
    CardDefinition {
        name: "Oni Possession",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::Trample],
            add_creature_types: vec![CreatureType::Demon, CreatureType::Spirit],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: R::Creature,
            },
        }],
        ..Default::default()
    }
}

/// Ragged Veins — {1}{B} Aura with flash. Whenever enchanted creature is dealt
/// damage, its controller loses that much life.
pub fn ragged_veins() -> CardDefinition {
    CardDefinition {
        name: "Ragged Veins",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::This))),
                    amount: Value::TriggerEventAmount,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Artifacts ────────────────────────────────────────────────────────────────

/// General's Kabuto — {4} Equipment. Equipped creature has shroud and all
/// combat damage dealt to it is prevented. Equip {2}.
pub fn generals_kabuto() -> CardDefinition {
    CardDefinition {
        name: "General's Kabuto",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Shroud],
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Prevent all combat damage that would be dealt to equipped creature.",
            effect: StaticEffect::PreventAllCombatDamageToAttached,
        }],
        ..Default::default()
    }
}

/// Hankyu — {1} Equipment. Equipped creature can bank aim counters on Hankyu and
/// spend the lot as damage. Equip {4}.
pub fn hankyu() -> CardDefinition {
    CardDefinition {
        name: "Hankyu",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![
                ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddCounter {
                        what: Selector::AttachmentGranting,
                        kind: CounterType::Aim,
                        amount: Value::ONE,
                    },
                    ..Default::default()
                },
                ActivatedAbility {
                    tap_cost: true,
                    // The removal is modeled as the first half of the
                    // resolution, so the damage still reads the counters
                    // removed this way.
                    effect: Effect::Seq(vec![
                        Effect::DealDamage {
                            to: Selector::Target(0),
                            amount: Value::CountersOn {
                                what: Box::new(Selector::AttachmentGranting),
                                kind: CounterType::Aim,
                            },
                        },
                        Effect::RemoveCounter {
                            what: Selector::AttachmentGranting,
                            kind: CounterType::Aim,
                            amount: Value::CountersOn {
                                what: Box::new(Selector::AttachmentGranting),
                                kind: CounterType::Aim,
                            },
                        },
                    ]),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Konda's Banner — {2} legendary Equipment attachable only to a legendary
/// creature. Creatures sharing a color or a creature type with the equipped
/// creature get +1/+1. Equip {2}.
pub fn kondas_banner() -> CardDefinition {
    let anthem = |filter: R| StaticAbility {
        description: "Creatures that share a color or creature type with equipped creature get +1/+1.",
        effect: StaticEffect::AnthemForFilter {
            filter: R::Creature.and(filter),
            power: 1,
            toughness: 1,
            keywords: vec![],
            opponents: false,
            all_players: true,
            only_your_turn: false,
            scale_by_counters_on_self: None,
        },
    };
    CardDefinition {
        name: "Konda's Banner",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        supertypes: legendary(),
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        attach_only_filter: Some(R::HasSupertype(Supertype::Legendary)),
        static_abilities: vec![
            anthem(R::SharesColorWithAttachedHost),
            anthem(R::SharesCreatureTypeWithAttachedHost),
        ],
        ..Default::default()
    }
}

/// Tenza, Godo's Maul — {3} legendary Equipment. Equipped creature gets +1/+1,
/// an extra +2/+2 while legendary, and trample while red. Equip {1}.
pub fn tenza_godos_maul() -> CardDefinition {
    CardDefinition {
        name: "Tenza, Godo's Maul",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        supertypes: legendary(),
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            conditional: vec![
                crate::card::ConditionalEquipBonus {
                    host_filter: R::HasSupertype(Supertype::Legendary),
                    power: 2,
                    toughness: 2,
                    ..Default::default()
                },
                crate::card::ConditionalEquipBonus {
                    host_filter: R::HasColor(Color::Red),
                    keywords: vec![Keyword::Trample],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Honor-Worn Shaku — {3} artifact. {T}: Add {C}. Tap an untapped legendary
/// permanent you control: Untap this artifact.
pub fn honor_worn_shaku() -> CardDefinition {
    CardDefinition {
        name: "Honor-Worn Shaku",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_other_filter: Some(
                    R::HasSupertype(Supertype::Legendary).and(R::ControlledByYou),
                ),
                effect: Effect::Untap {
                    what: Selector::This,
                    up_to: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Hair-Strung Koto — {6} artifact. Tap an untapped creature you control:
/// Target player mills a card.
pub fn hair_strung_koto() -> CardDefinition {
    CardDefinition {
        name: "Hair-Strung Koto",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Creature.and(R::ControlledByYou)),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Imi Statue — {3} artifact. Players can't untap more than one artifact during
/// their untap steps.
pub fn imi_statue() -> CardDefinition {
    CardDefinition {
        name: "Imi Statue",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Players can't untap more than one artifact during their untap steps.",
            effect: StaticEffect::MaxOneUntapPerStep { filter: R::Artifact },
        }],
        ..Default::default()
    }
}

/// Orochi Hatchery — {X}{X} artifact entering with X charge counters. {5}, {T}:
/// Create a 1/1 green Snake for each charge counter.
pub fn orochi_hatchery() -> CardDefinition {
    CardDefinition {
        name: "Orochi Hatchery",
        cost: cost(&[crate::mana::x(), crate::mana::x()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Charge, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Charge,
                },
                definition: Box::new(snake_token()),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Night Dealings — {2}{B}{B} enchantment. Banks a theft counter per point of
/// damage your sources deal a player; {2}{B}{B}, remove X: tutor a nonland card
/// with mana value X.
pub fn night_dealings() -> CardDefinition {
    CardDefinition {
        name: "Night Dealings",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::YourSourceDamagedOpponent),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Theft,
                amount: Value::TriggerEventAmount,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b(), b()]),
            remove_counter_x: Some(CounterType::Theft),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Nonland.and(R::ManaValueExactlyXFromCost),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Night of Souls' Betrayal — {2}{B}{B} legendary enchantment. All creatures get
/// -1/-1.
pub fn night_of_souls_betrayal() -> CardDefinition {
    CardDefinition {
        name: "Night of Souls' Betrayal",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        supertypes: legendary(),
        static_abilities: vec![StaticAbility {
            description: "All creatures get -1/-1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature),
                power: -1,
                toughness: -1,
            },
        }],
        ..Default::default()
    }
}

/// Blood Rites — {3}{R}{R} enchantment. {1}{R}, Sacrifice a creature: deals 2
/// damage to any target.
pub fn blood_rites() -> CardDefinition {
    CardDefinition {
        name: "Blood Rites",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nature's Will — {2}{G}{G} enchantment. Whenever your creatures deal combat
/// damage to a player, tap all their lands and untap all yours.
pub fn natures_will() -> CardDefinition {
    CardDefinition {
        name: "Nature's Will",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: Selector::EachPermanent(R::Land.and(R::ControlledByOpponent)),
                },
                Effect::Untap {
                    what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                    up_to: None,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Vassal's Duty — {3}{W} enchantment. {1}: The next 1 damage that would be
/// dealt to target legendary creature you control this turn hits you instead.
pub fn vassals_duty() -> CardDefinition {
    CardDefinition {
        name: "Vassal's Duty",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::RedirectNextDamage {
                target: target_filtered(
                    R::Creature
                        .and(R::HasSupertype(Supertype::Legendary))
                        .and(R::ControlledByYou),
                ),
                to: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Blood Speaker — {3}{B} 3/2. Upkeep: may sacrifice it to tutor a Demon.
/// A Demon entering returns it from your graveyard to your hand.
pub fn blood_speaker() -> CardDefinition {
    CardDefinition {
        name: "Blood Speaker",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Ogre, CreatureType::Shaman]),
        power: 3,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::MayDo {
                    description: "Sacrifice Blood Speaker to search for a Demon".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Sacrifice {
                            who: Selector::You,
                            count: Value::ONE,
                            filter: R::HasName("Blood Speaker".into()),
                        },
                        Effect::Search {
                            who: PlayerRef::You,
                            filter: R::HasCreatureType(CreatureType::Demon),
                            to: ZoneDest::Hand(PlayerRef::You),
                        },
                    ])),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::EntersBattlefield,
                    EventScope::FromYourGraveyard,
                )
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Demon),
                }),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
        ],
        ..Default::default()
    }
}

/// Bloodthirsty Ogre — {2}{B} 3/1. {T}: bank a devotion counter. {T}: target
/// creature gets -X/-X, X = devotion counters; needs a Demon.
pub fn bloodthirsty_ogre() -> CardDefinition {
    CardDefinition {
        name: "Bloodthirsty Ogre",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![
            CreatureType::Ogre,
            CreatureType::Warrior,
            CreatureType::Shaman,
        ]),
        power: 3,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Devotion,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Demon).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                }),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Times(
                        Box::new(Value::Const(-1)),
                        Box::new(Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Devotion,
                        }),
                    ),
                    toughness: Value::Times(
                        Box::new(Value::Const(-1)),
                        Box::new(Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Devotion,
                        }),
                    ),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Pious Kitsune — {2}{W} 1/2. Upkeep: bank a devotion counter, then gain 1 life
/// per counter if Eight-and-a-Half-Tails is out. {T}, remove one: gain 1 life.
pub fn pious_kitsune() -> CardDefinition {
    CardDefinition {
        name: "Pious Kitsune",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Fox, CreatureType::Cleric]),
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Devotion,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(R::HasName(
                            "Eight-and-a-Half-Tails".into(),
                        )),
                        n: Value::ONE,
                    },
                    then: Box::new(Effect::GainLife {
                        who: Selector::You,
                        amount: Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Devotion,
                        },
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Devotion, 1)),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ore Gorger — {3}{R}{R} 3/1 Spirit. Whenever you cast a Spirit or Arcane
/// spell, you may destroy target nonbasic land.
pub fn ore_gorger() -> CardDefinition {
    CardDefinition {
        name: "Ore Gorger",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Spirit]),
        power: 3,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Spirit)
                        .or(R::HasSpellSubtype(SpellSubtype::Arcane)),
                },
            ),
            effect: Effect::MayDo {
                description: "Destroy target nonbasic land".into(),
                body: Box::new(Effect::Destroy {
                    what: target_filtered(R::IsNonbasicLand),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Rootrunner — {2}{G}{G} 3/3 Spirit with soulshift 3. {G}{G}, Sacrifice this:
/// Put target land on top of its owner's library.
pub fn rootrunner() -> CardDefinition {
    CardDefinition {
        name: "Rootrunner",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Spirit]),
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), g()]),
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Land),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::Top,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Eight-and-a-Half-Tails — {W}{W} 2/2. {1}{W}: target permanent you control
/// gains protection from white. {1}: target spell or permanent becomes white.
pub fn eight_and_a_half_tails() -> CardDefinition {
    CardDefinition {
        name: "Eight-and-a-Half-Tails",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Fox, CreatureType::Cleric]),
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w()]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                    keyword: Keyword::Protection(Color::White),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::BecomeColor {
                    what: target_filtered(R::Permanent.or(R::IsSpellOnStack)),
                    colors: vec![Color::White],
                    duration: Duration::EndOfTurn,
                    additive: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Spells ───────────────────────────────────────────────────────────────────

/// Candles' Glow — {1}{W} Instant — Arcane. Prevent the next 3 damage to any
/// target this turn; gain that much life. Splice onto Arcane {1}{W}.
pub fn candles_glow() -> CardDefinition {
    CardDefinition {
        name: "Candles' Glow",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        keywords: vec![Keyword::Splice(cost(&[generic(1), w()]), SpellSubtype::Arcane)],
        effect: Effect::PreventNextDamageAndGainLife {
            target: Selector::Target(0),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Cranial Extraction — {3}{B} Sorcery — Arcane. Name a nonland card; exile
/// every copy from target player's graveyard, hand, and library.
pub fn cranial_extraction() -> CardDefinition {
    CardDefinition {
        name: "Cranial Extraction",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: arcane(),
        effect: Effect::NameCardExileMatchingAllZones,
        ..Default::default()
    }
}

/// Devouring Rage — {4}{R} Instant — Arcane. Target creature gets +3/+0, plus an
/// extra +3/+0 per Spirit sacrificed as an additional cost.
pub fn devouring_rage() -> CardDefinition {
    CardDefinition {
        name: "Devouring Rage",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificeAnyNumber {
            filter: R::HasCreatureType(CreatureType::Spirit),
        }],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Sum(vec![
                Value::Const(3),
                Value::Times(Box::new(Value::Const(3)), Box::new(Value::SacrificedCount)),
            ]),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Feast of Worms — {3}{G}{G} Sorcery — Arcane. Destroy target land; a legendary
/// one also costs its controller another land.
pub fn feast_of_worms() -> CardDefinition {
    CardDefinition {
        name: "Feast of Worms",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Sorcery],
        subtypes: arcane(),
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasSupertype(Supertype::Legendary),
                },
                then: Box::new(Effect::Seq(vec![
                    Effect::Destroy {
                        what: Selector::Target(0),
                    },
                    Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(
                            0,
                        )))),
                        count: Value::ONE,
                        filter: R::Land,
                    },
                ])),
                else_: Box::new(Effect::Destroy {
                    what: Selector::Target(0),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Hisoka's Defiance — {1}{U} Instant. Counter target Spirit or Arcane spell.
pub fn hisokas_defiance() -> CardDefinition {
    CardDefinition {
        name: "Hisoka's Defiance",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(R::IsSpellOnStack.and(
                R::HasCreatureType(CreatureType::Spirit).or(R::HasSpellSubtype(SpellSubtype::Arcane)),
            )),
        },
        ..Default::default()
    }
}

/// Thoughtbind — {2}{U} Instant. Counter target spell with mana value 4 or less.
pub fn thoughtbind() -> CardDefinition {
    CardDefinition {
        name: "Thoughtbind",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(R::IsSpellOnStack.and(R::ManaValueAtMost(4))),
        },
        ..Default::default()
    }
}

/// Sideswipe — {1}{R} Instant. You may change any targets of target Arcane
/// spell.
pub fn sideswipe() -> CardDefinition {
    CardDefinition {
        name: "Sideswipe",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseNewTargetsForSpell {
            what: target_filtered(
                R::IsSpellOnStack.and(R::HasSpellSubtype(SpellSubtype::Arcane)),
            ),
        },
        ..Default::default()
    }
}

/// Mana Seism — {1}{R} Sorcery. Sacrifice any number of lands, then add that
/// much {C}.
pub fn mana_seism() -> CardDefinition {
    CardDefinition {
        name: "Mana Seism",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificeAnyNumber {
            filter: R::Land.and(R::ControlledByYou),
        }],
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colorless(Value::SacrificedCount),
        },
        ..Default::default()
    }
}

/// Reverse the Sands — {6}{W}{W} Sorcery. Redistribute players' life totals
/// (two-player: exchange them).
pub fn reverse_the_sands() -> CardDefinition {
    CardDefinition {
        name: "Reverse the Sands",
        cost: cost(&[generic(6), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ExchangeLifeTotals {
            a: Selector::You,
            b: Selector::Player(PlayerRef::EachOpponent),
        },
        ..Default::default()
    }
}

/// Soulblast — {3}{R}{R}{R} Instant. Sacrifice all your creatures as an
/// additional cost; deals damage equal to their total power to any target.
pub fn soulblast() -> CardDefinition {
    CardDefinition {
        name: "Soulblast",
        cost: cost(&[generic(3), r(), r(), r()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificeAll {
            filter: R::Creature.and(R::ControlledByYou),
        }],
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::SacrificedTotalPower,
        },
        ..Default::default()
    }
}

/// Tide of War — {4}{R}{R} enchantment. Whenever creatures block, a coin flip
/// sacrifices either every blocking or every blocked creature.
pub fn tide_of_war() -> CardDefinition {
    CardDefinition {
        name: "Tide of War",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_turn: false,
                ..EventSpec::new(EventKind::BlocksNOrMore(1), EventScope::AnyPlayer)
            },
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::SacrificeAllMatching {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    filter: R::Creature.and(R::IsBlocking),
                }),
                on_tails: Box::new(Effect::SacrificeAllMatching {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    filter: R::Creature.and(R::IsBlocked),
                }),
            },
        }],
        ..Default::default()
    }
}


/// Junkyo Bell — {4} artifact. Upkeep: you may pump a creature by the size of
/// your board, at the price of sacrificing it at the next end step.
pub fn junkyo_bell() -> CardDefinition {
    CardDefinition {
        name: "Junkyo Bell",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Pump a creature, then sacrifice it at end of turn".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::Creature.and(R::ControlledByYou)),
                        power: Value::count(Selector::EachPermanent(
                            R::Creature.and(R::ControlledByYou),
                        )),
                        toughness: Value::count(Selector::EachPermanent(
                            R::Creature.and(R::ControlledByYou),
                        )),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::AtNextEndStep {
                        body: Box::new(Effect::Destroy {
                            what: Selector::Target(0),
                        }),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Kusari-Gama — {3} Equipment. Equipped creature can pump itself, and damage it
/// deals to a blocker splashes onto the rest of the defending player's board.
/// Equip {3}.
pub fn kusari_gama() -> CardDefinition {
    CardDefinition {
        name: "Kusari-Gama",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            }],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToCreature,
                    EventScope::SelfSource,
                ),
                // "Each other creature defending player controls" — read as the
                // opponent's non-blocking creatures (exact at two players).
                effect: Effect::DealDamage {
                    to: Selector::EachPermanent(
                        R::Creature
                            .and(R::ControlledByOpponent)
                            .and(R::Not(Box::new(R::IsBlocking))),
                    ),
                    amount: Value::TriggerEventAmount,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Oathkeeper, Takeno's Daisho — {3} legendary Equipment. +3/+1; a Samurai that
/// dies while equipped comes back under your control, and the Equipment exiles
/// its host when it hits the graveyard. Equip {2}.
pub fn oathkeeper_takenos_daisho() -> CardDefinition {
    CardDefinition {
        name: "Oathkeeper, Takeno's Daisho",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        supertypes: legendary(),
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 1,
            triggers_on_equipment: true,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Samurai),
                    },
                ),
                effect: Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            }],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                to: ZoneDest::Exile,
            },
        }],
        ..Default::default()
    }
}

/// Shell of the Last Kappa — {3} legendary artifact. {3}, {T}: exile a spell
/// that targets you; {3}, {T}, sacrifice: free-cast one of the cards it took.
pub fn shell_of_the_last_kappa() -> CardDefinition {
    CardDefinition {
        name: "Shell of the Last Kappa",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        supertypes: legendary(),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::CounterSpellToZone {
                    what: target_filtered(
                        R::IsSpellOnStack
                            .and(
                                R::HasCardType(CardType::Instant)
                                    .or(R::HasCardType(CardType::Sorcery)),
                            )
                            .and(R::SpellTargetsControllerOrControlled),
                    ),
                    zone: crate::effect::CounteredSpellZone::ExileWithSource,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::CastWithoutPayingImmediate {
                    reduce_generic: 0,
                                pay_own_cost: false,
                    what: Selector::CardExiledWithSource,
                    source_zone: crate::card::Zone::Exile,
                    exile_after: false,
                    copy: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Petals of Insight — {4}{U} Sorcery — Arcane. Look at the top three; bottom
/// them to return this to hand, or leave them and draw three.
pub fn petals_of_insight() -> CardDefinition {
    CardDefinition {
        name: "Petals of Insight",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: arcane(),
        effect: Effect::LookTopMayBottomAllElse {
            who: None,
            count: Value::Const(3),
            then: Box::new(Effect::ReturnResolvingSpellToHand),
            else_: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            }),
        },
        ..Default::default()
    }
}

/// Cut the Tethers — {2}{U}{U} Sorcery. Every Spirit goes home unless its
/// controller pays {3}.
pub fn cut_the_tethers() -> CardDefinition {
    CardDefinition {
        name: "Cut the Tethers",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ReturnEachUnlessPays {
            filter: R::HasCreatureType(CreatureType::Spirit),
            cost: cost(&[generic(3)]),
        },
        ..Default::default()
    }
}

/// Uba Mask — {4} artifact. Draws become face-up exiles their owner may play
/// that turn.
pub fn uba_mask() -> CardDefinition {
    CardDefinition {
        name: "Uba Mask",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "If a player would draw a card, that player exiles that card face up instead.",
            effect: StaticEffect::PlayersDrawExiledPlayable,
        }],
        ..Default::default()
    }
}

/// Tatsumasa, the Dragon's Fang — {6} legendary Equipment. +5/+5; {6}, exile it:
/// mint a 5/5 flying Dragon Spirit that gives it back when it dies. Equip {3}.
pub fn tatsumasa_the_dragons_fang() -> CardDefinition {
    CardDefinition {
        name: "Tatsumasa, the Dragon's Fang",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        supertypes: legendary(),
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 5,
            toughness: 5,
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            exile_self_cost: true,
            effect: Effect::CreateTokenReturnSelfWhenItDies {
                definition: Box::new(TokenDefinition {
                    name: "Dragon Spirit".into(),
                    power: 5,
                    toughness: 5,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Blue],
                    keywords: vec![Keyword::Flying],
                    subtypes: types(vec![CreatureType::Dragon, CreatureType::Spirit]),
                    ..Default::default()
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nezumi Shortfang // Stabwhisker the Odious — {1}{B} 1/1 Rat Rogue. Its
/// discard flips it once the victim is empty-handed; Stabwhisker then bleeds
/// each opponent for every card short of three.
pub fn nezumi_shortfang() -> CardDefinition {
    let stabwhisker = CardDefinition {
        name: "Stabwhisker the Odious",
        card_types: vec![CardType::Creature],
        supertypes: legendary(),
        subtypes: types(vec![CreatureType::Rat, CreatureType::Shaman]),
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::OpponentControl,
            ),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Max(
                    Box::new(Value::ZERO),
                    Box::new(Value::Sum(vec![
                        Value::Const(3),
                        Value::Times(
                            Box::new(Value::Const(-1)),
                            Box::new(Value::CardsInHandMatching {
                                who: PlayerRef::ActivePlayer,
                                filter: R::Any,
                            }),
                        ),
                    ])),
                ),
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Nezumi Shortfang",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Rat, CreatureType::Rogue]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::OpponentPlayer,
                    },
                    amount: Value::ONE,
                    random: false,
                },
                Effect::If {
                    cond: Predicate::ValueAtMost(
                        Value::CardsInHandMatching {
                            who: PlayerRef::Target(0),
                            filter: R::Any,
                        },
                        Value::ZERO,
                    ),
                    then: Box::new(Effect::Flip {
                        what: Selector::This,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        flip_face: Some(Box::new(stabwhisker)),
        ..Default::default()
    }
}
