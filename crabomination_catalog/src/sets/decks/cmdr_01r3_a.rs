//! Commander: the 2026-10-21 (`01R3xh4r`) one- to four-gap pair closers. Tests in
//! `tests/recent_b/cmdr_01r3_a.rs`.

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, BattleSubtype, CardDefinition, CardType, ConditionalEquipBonus,
    CounterType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    MayPlayDuration, Prototype, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Zone,
};
use crate::effect::shortcut::{craft, freerunning, magecraft, partner_with_search, support, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, LibraryPosition, LookExileGrant, PlayerRef, Predicate, Selector, Value,
    ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w, x};
use std::sync::Arc;

/// Arcane Proxy — {7} 4/3 artifact creature Wizard; prototype {1}{U}{U} 2/1
/// (CR 702.160). When it enters, if you cast it, exile target instant or
/// sorcery card with mana value <= its power from your graveyard, copy it, and
/// you may cast the copy without paying its mana cost (CR 707.12).
pub fn arcane_proxy() -> CardDefinition {
    let spell = R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        name: "Arcane Proxy",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        power: 4,
        toughness: 3,
        prototype: Some(Box::new(Prototype { cost: cost(&[generic(1), u(), u()]), power: 2, toughness: 1 })),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SourceWasCast),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(spell.and(R::InYourGraveyard).and(R::ManaValueAtMostSourcePower)),
                    to: ZoneDest::Exile,
                },
                Effect::CastWithoutPayingImmediate {
                    reduce_generic: 0,
                    pay_own_cost: false,
                    what: Selector::LastMoved,
                    source_zone: Zone::Exile,
                    exile_after: false,
                    copy: true,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Worldpurge — {4}{W/U}{W/U}{W/U}{W/U} sorcery. Return all permanents to
/// their owners' hands; each player keeps up to seven cards in hand and
/// shuffles the rest into their library; each player loses all unspent mana.
pub fn worldpurge() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    CardDefinition {
        name: "Worldpurge",
        cost: cost(&[generic(4), wu(), wu(), wu(), wu()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move { what: Selector::EachPermanent(R::Any), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            Effect::EachPlayerKeepsUpToFromHandShufflesRest { keep: 7 },
            Effect::LoseUnspentMana { who: PlayerRef::EachPlayer },
        ]),
        ..Default::default()
    }
}

/// Braided Net // Braided Quipu — {2}{U} artifact; enters with three net
/// counters. {T}, remove a net counter: tap another target nonland permanent;
/// its activated abilities can't be activated for as long as it remains tapped
/// (CR 611.2c, `Duration::WhileTapped`). Craft with an artifact {1}{U}.
/// Braided Quipu: {3}{U}, {T}: draw a card for each artifact you control, then
/// put it into its owner's library third from the top.
pub fn braided_net() -> CardDefinition {
    let quipu = CardDefinition {
        name: "Braided Quipu",
        card_types: vec![CardType::Artifact],
        color_indicator: vec![Color::Blue],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)))),
                },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::This)), pos: LibraryPosition::FromTop(2) },
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    };
    let other = R::Permanent.and(R::Land.negate()).and(R::OtherThanSource);
    CardDefinition {
        name: "Braided Net",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Net, Value::Const(3))),
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Net, 1)),
                effect: Effect::Seq(vec![
                    Effect::Tap { what: target_filtered(other) },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::CantActivateAbilities,
                        duration: Duration::WhileTapped,
                    },
                ]),
                ..Default::default()
            },
            craft(cost(&[generic(1), u()]), R::Artifact, 1),
        ],
        back_face: Some(Box::new(quipu)),
        ..Default::default()
    }
}

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

/// Duskrider Peregrine — {5}{W} 3/3 Bird; flying, protection from black,
/// suspend 3—{1}{W}.
pub fn duskrider_peregrine() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Protection(Color::Black),
            Keyword::Suspend(3, cost(&[generic(1), w()])),
        ],
        ..creature("Duskrider Peregrine", cost(&[generic(5), w()]), vec![CreatureType::Bird], 3, 3)
    }
}

/// Deem Worthy — {4}{R} instant: 7 damage to target creature. Cycling
/// {3}{R}; when you cycle it, you may have it deal 2 damage to target creature
/// (CR 702.29c).
pub fn deem_worthy() -> CardDefinition {
    CardDefinition {
        name: "Deem Worthy",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(3), r()]))],
        effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(7) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Deal 2 damage to target creature?".into(),
                body: Box::new(Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(2) }),
            },
        }],
        ..Default::default()
    }
}

/// Dire Wolves — {2}{G} 2/2 Wolf; banding as long as you control a Plains.
pub fn dire_wolves() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has banding as long as you control a Plains.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Banding,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::HasLandType(LandType::Plains).and(R::ControlledByYou)),
                    n: Value::ONE,
                },
            },
        }],
        ..creature("Dire Wolves", cost(&[generic(2), g()]), vec![CreatureType::Wolf], 2, 2)
    }
}

/// Nature's Blessing — {2}{G}{W} enchantment. {G}{W}, discard a card: put a
/// +1/+1 counter on target creature, or it gains banding, first strike or
/// trample indefinitely.
pub fn natures_blessing() -> CardDefinition {
    let creature = || target_filtered(R::Creature);
    let gain = |keyword| Effect::GrantKeyword { what: creature(), keyword, duration: Duration::Permanent };
    CardDefinition {
        name: "Nature's Blessing",
        cost: cost(&[generic(2), g(), w()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), w()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::ChooseMode(vec![
                Effect::AddCounter { what: creature(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                gain(Keyword::Banding),
                gain(Keyword::FirstStrike),
                gain(Keyword::Trample),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Disciple of Caelus Nin — {4}{W} 3/4 Human Wizard. When it enters, starting
/// with you, each player chooses up to five permanents they control; every
/// other permanent but this one phases out. Permanents can't phase in.
pub fn disciple_of_caelus_nin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::EachPlayerKeepsUpToPermanentsRestPhaseOut { keep: 5 },
        }],
        static_abilities: vec![StaticAbility {
            description: "Permanents can't phase in.",
            effect: StaticEffect::PermanentsCantPhaseIn,
        }],
        ..creature(
            "Disciple of Caelus Nin",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Geistflame Reservoir — {2}{R} artifact. Whenever you cast an instant or
/// sorcery spell, put a charge counter on it. {1}{R}, {T}, remove any number
/// of charge counters: that much damage to any target. {1}{R}, {T}: exile the
/// top card of your library; you may play it this turn.
pub fn geistflame_reservoir() -> CardDefinition {
    CardDefinition {
        name: "Geistflame Reservoir",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Charge,
            amount: Value::ONE,
        })],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                tap_cost: true,
                remove_counter_x: Some(CounterType::Charge),
                effect: Effect::DealDamage { to: target_filtered(R::Any), amount: Value::XFromCost },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                tap_cost: true,
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Invasion of Arcavios // Invocation of the Founders — {3}{U}{U} Siege,
/// defense 7. When it enters, search your library and/or graveyard for an
/// instant or sorcery card you own and put it into your hand (outside the game
/// holds nothing in Commander, CR 903.11). Back: whenever you cast an instant
/// or sorcery spell from your hand, you may copy it and choose new targets.
pub fn invasion_of_arcavios() -> CardDefinition {
    let spell = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    let founders = CardDefinition {
        name: "Invocation of the Founders",
        card_types: vec![CardType::Enchantment],
        color_indicator: vec![Color::Blue],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellMatches(spell()),
                Predicate::CastSpellFromHand,
            ])),
            effect: Effect::MayDo {
                description: "Copy that spell?".into(),
                body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE }),
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Arcavios",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![BattleSubtype::Siege], ..Default::default() },
        defense: 7,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::SearchZones {
                who: PlayerRef::You,
                zones: vec![Zone::Library, Zone::Graveyard],
                filter: spell(),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        back_face: Some(Box::new(founders)),
        ..Default::default()
    }
}

/// Fungal Behemoth — {3}{G} */* Fungus; P/T = +1/+1 counters on creatures you
/// control. Suspend X—{X}{G}{G}, X can't be 0. Whenever a time counter is
/// removed from it while it's exiled, you may put a +1/+1 counter on target
/// creature.
pub fn fungal_behemoth() -> CardDefinition {
    let counters = || Value::CountersOn {
        what: Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))),
        kind: CounterType::PlusOnePlusOne,
    };
    CardDefinition {
        keywords: vec![Keyword::Suspend(0, cost(&[x(), g(), g()]))],
        static_abilities: vec![StaticAbility {
            description: "P/T equal to the +1/+1 counters on creatures you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: counters(), toughness: counters() },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterRemoved(CounterType::Time), EventScope::SelfSource).while_suspended(),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on target creature?".into(),
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature("Fungal Behemoth", cost(&[generic(3), g()]), vec![CreatureType::Fungus], 0, 0)
    }
}

/// Feral Encounter — {G}{G} sorcery. Look at the top five; you may exile a
/// creature card among them and cast it this turn; the rest go to the bottom
/// at random. At the beginning of the next combat this turn, target creature
/// you control deals damage equal to its power to up to one target creature
/// you don't control.
pub fn feral_encounter() -> CardDefinition {
    CardDefinition {
        name: "Feral Encounter",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LookTopExileOneMayPlay {
                count: Value::Const(5),
                who: PlayerRef::You,
                grant: LookExileGrant::CreatureMayThisTurn,
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::NextCombatThisTurn,
                body: Box::new(Effect::OptionalTargets {
                    min: 1,
                    body: Box::new(Effect::DealDamageEqualToPower {
                        source: target_filtered(R::Creature.and(R::ControlledByYou)),
                        target: target_filtered(R::Creature.and(R::ControlledByYou.negate())),
                    }),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Vedalken Squirrel-Whacker — {3}{U} */* Vedalken Guest. As it enters, roll
/// a d6 twice: base power the first result, base toughness the second. If you
/// would roll one or more d6, you may exchange one result with its base power
/// or base toughness (CR 706.6).
pub fn vedalken_squirrel_whacker() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::RollDiceSetBasePt { sides: 6 }),
        static_abilities: vec![StaticAbility {
            description: "Exchange a d6 result with this creature's base power or toughness.",
            effect: StaticEffect::MayExchangeD6WithSelfBasePt,
        }],
        ..creature(
            "Vedalken Squirrel-Whacker",
            cost(&[generic(3), u()]),
            vec![CreatureType::Vedalken, CreatureType::Guest],
            0,
            0,
        )
    }
}

/// Soul Echo — {X}{W}{W} enchantment; enters with X echo counters. You don't
/// lose the game for having 0 or less life. At the beginning of your upkeep,
/// sacrifice it with no echo counters on it; otherwise target opponent may
/// choose that each 1 damage dealt to you until your next upkeep removes an
/// echo counter from it instead (CR 614.1a).
pub fn soul_echo() -> CardDefinition {
    let echoes = Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Echo };
    CardDefinition {
        name: "Soul Echo",
        cost: cost(&[x(), w(), w()]),
        card_types: vec![CardType::Enchantment],
        enters_with_counters: Some((CounterType::Echo, Value::XFromCost)),
        static_abilities: vec![
            StaticAbility {
                description: "You don't lose the game for having 0 or less life.",
                effect: StaticEffect::ControllerDoesntLoseForZeroLife,
            },
            StaticAbility {
                description: "Damage to you removes echo counters instead while chosen.",
                effect: StaticEffect::ReplaceDamageToYouRemovingStampedCounters { kind: CounterType::Echo },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::ValueAtMost(echoes, Value::Const(0)),
                then: Box::new(Effect::SacrificeSource),
                else_: Box::new(Effect::PlayerMayStampSourceTurn { who: target_filtered(R::OpponentPlayer) }),
            },
        }],
        ..Default::default()
    }
}

/// Mindlink Mech — {2}{U} 4/3 Vehicle, flying, crew 1. Whenever it becomes
/// crewed for the first time each turn, until end of turn it becomes a copy of
/// target nonlegendary creature that crewed it this turn, except it's 4/3, a
/// Vehicle artifact in addition to its other types, and has flying (CR 707.9b).
pub fn mindlink_mech() -> CardDefinition {
    let mut crewed = EventSpec::new(EventKind::CrewsOrSaddles { main_phase_only: false }, EventScope::YourControl)
        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSource });
    crewed.once_per_turn = true;
    let crew = R::Creature.and(R::CrewedSourceThisTurn).and(R::HasSupertype(Supertype::Legendary).negate());
    CardDefinition {
        name: "Mindlink Mech",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: crewed,
            effect: Effect::Seq(vec![
                Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: target_filtered(crew),
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                },
                Effect::AmendCopy {
                    what: Selector::This,
                    keep_activated: vec![],
                    keep_triggered: vec![],
                    pt: Some((4, 3)),
                    keep_name: false,
                    legendary: false,
                    keywords: vec![Keyword::Flying],
                    card_types: vec![CardType::Artifact],
                    creature_types: vec![],
                },
                // Layer 4; indefinite is harmless, the Mech is a Vehicle anyway.
                Effect::SetArtifactSubtypes { what: Selector::This, subtypes: vec![ArtifactSubtype::Vehicle] },
            ]),
        }],
        ..Default::default()
    }
}

/// Encrust — {1}{U}{U} Aura, enchant artifact or creature. Enchanted permanent
/// doesn't untap during its controller's untap step and its activated
/// abilities can't be activated.
pub fn encrust() -> CardDefinition {
    CardDefinition {
        name: "Encrust",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Artifact.or(R::Creature)) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted permanent doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::AttachedTo(Box::new(Selector::This)) },
        }],
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::CantActivateAbilities], ..Default::default() }),
        ..Default::default()
    }
}

/// Ice Floe — land. You may choose not to untap it during your untap step.
/// {T}: tap target creature without flying that's attacking you; it doesn't
/// untap during its controller's untap step for as long as this land remains
/// tapped.
pub fn ice_floe() -> CardDefinition {
    CardDefinition {
        name: "Ice Floe",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::TapAndUntapLock {
                what: target_filtered(R::Creature.and(R::IsAttackingYou).and(R::HasKeyword(Keyword::Flying).negate())),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lifecraft Awakening — {X}{G} instant. Put X +1/+1 counters on target
/// artifact you control; if it isn't a creature or Vehicle, it becomes a 0/0
/// Construct artifact creature.
pub fn lifecraft_awakening() -> CardDefinition {
    let artifact = || target_filtered(R::Artifact.and(R::ControlledByYou));
    CardDefinition {
        name: "Lifecraft Awakening",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter { what: artifact(), kind: CounterType::PlusOnePlusOne, amount: Value::XFromCost },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                },
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::BecomeCreature {
                    what: artifact(),
                    power: Value::Const(0),
                    toughness: Value::Const(0),
                    creature_types: vec![CreatureType::Construct],
                    keywords: vec![],
                    duration: Duration::Permanent,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Deadly Wanderings — {3}{B}{B} enchantment. As long as you control exactly
/// one creature, it gets +2/+0 and has deathtouch and lifelink.
pub fn deadly_wanderings() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Deadly Wanderings",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "As long as you control exactly one creature, it gets +2/+0 and has deathtouch and lifelink.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::ValueEquals(Value::count(yours()), Value::ONE),
                applies_to: yours(),
                power: 2,
                toughness: 0,
                keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
            },
        }],
        ..Default::default()
    }
}

/// Grotesque Mutation — {1}{B} instant: target creature gets +3/+1 and gains
/// lifelink until end of turn.
pub fn grotesque_mutation() -> CardDefinition {
    CardDefinition {
        name: "Grotesque Mutation",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Monastery Raid — {2}{R} sorcery, freerunning {X}{R} (CR 702.173). Exile the
/// top two cards of your library — the top X if the freerunning cost was paid —
/// and you may play them until the end of your next turn.
pub fn monastery_raid() -> CardDefinition {
    let raid = |count| Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count,
        duration: MayPlayDuration::EndOfControllersNextTurn,
        pay_any_color: false,
        max_mana_value: None,
        pay_own_cost: true,
        uncast_penalty: None,
    };
    CardDefinition {
        name: "Monastery Raid",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: raid(Value::Const(2)),
        alternative_cost: Some(AlternativeCost {
            effect_override: Some(raid(Value::XFromCost)),
            ..freerunning(cost(&[x(), r()]))
        }),
        ..Default::default()
    }
}

/// Slayer's Plate — {3} Equipment: +4/+2. Whenever equipped creature dies, if
/// it was a Human, create a 1/1 white Spirit creature token with flying. Equip
/// {3}. The trigger rides the Plate (`EnchantedBySource` reads the death-time
/// attachment), so the Spirit is the Plate controller's.
pub fn slayers_plate() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Slayer's Plate",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 4, toughness: 2, ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCreatureType(CreatureType::Human) },
            ),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(spirit) },
        }],
        ..Default::default()
    }
}

/// Braids's Frightful Return — {2}{B} Saga, read ahead (CR 714.3c). I: you may
/// sacrifice a creature; if you do, each opponent discards a card. II: return
/// target creature card from your graveyard to your hand. III: target opponent
/// may sacrifice a nonland, nontoken permanent; if they don't, they lose 2 life
/// and you draw a card.
pub fn braidss_frightful_return() -> CardDefinition {
    CardDefinition {
        name: "Braids's Frightful Return",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        read_ahead: true,
        saga_chapters: vec![
            (
                1,
                Effect::MaySacrifice {
                    description: "Sacrifice a creature to make each opponent discard?".into(),
                    filter: R::Creature,
                    count: Value::ONE,
                    then: Box::new(Effect::Discard {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::ONE,
                        random: false,
                    }),
                    else_: None,
                },
            ),
            (
                2,
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ),
            (
                3,
                Effect::Punisher {
                    chooser: target_filtered(R::OpponentPlayer),
                    options: vec![Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::You),
                        count: Value::ONE,
                        filter: R::Permanent.and(R::Land.negate()).and(R::IsToken.negate()),
                    }],
                    otherwise: Box::new(Effect::Seq(vec![
                        Effect::LoseLife { who: Selector::Target(0), amount: Value::Const(2) },
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ])),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Kor Dirge — {2}{B} instant. All damage that would be dealt this turn to
/// target creature you control by a source of your choice is dealt to another
/// target creature instead (CR 614.9).
pub fn kor_dirge() -> CardDefinition {
    CardDefinition {
        name: "Kor Dirge",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PreventNextDamageFromChosenSource {
            filter: R::Any,
            reflect: false,
            to: Some(target_filtered(R::Creature.and(R::ControlledByYou))),
            gain_life: false,
            redirect_to: Some(Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::OtherThanTargetSlot(0)) }),
            whole_turn: true,
            exile_top_per_prevented: false,
        },
        ..Default::default()
    }
}

/// Put Away — {2}{U}{U} instant. Counter target spell. You may shuffle up to
/// one target card from your graveyard into your library.
pub fn put_away() -> CardDefinition {
    CardDefinition {
        name: "Put Away",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
                Effect::Move {
                    what: Selector::TargetFiltered { slot: 1, filter: R::InYourGraveyard },
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Shuffled },
                },
            ])),
        },
        ..Default::default()
    }
}

/// Ceremonious Rejection — {U} instant: counter target colorless spell.
pub fn ceremonious_rejection() -> CardDefinition {
    CardDefinition {
        name: "Ceremonious Rejection",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::Colorless)) },
        ..Default::default()
    }
}

/// Deny Entry — {2}{U} instant: counter target creature spell, then draw a
/// card and discard a card.
pub fn deny_entry() -> CardDefinition {
    CardDefinition {
        name: "Deny Entry",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::Creature)) },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        ]),
        ..Default::default()
    }
}

/// Kiss of the Amesha — {4}{W}{U} sorcery: target player gains 7 life and
/// draws two cards.
pub fn kiss_of_the_amesha() -> CardDefinition {
    CardDefinition {
        name: "Kiss of the Amesha",
        cost: cost(&[generic(4), w(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainLife { who: target_filtered(R::Player), amount: Value::Const(7) },
            Effect::Draw { who: Selector::Target(0), amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Symbol of Unsummoning — {2}{U} sorcery: return target creature to its
/// owner's hand; draw a card.
pub fn symbol_of_unsummoning() -> CardDefinition {
    CardDefinition {
        name: "Symbol of Unsummoning",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::Creature), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Command of Unsummoning — {2}{U} instant; cast only during the declare
/// attackers step if you've been attacked this step. Return one or two target
/// attacking creatures to their owner's hand.
pub fn command_of_unsummoning() -> CardDefinition {
    let attacker = |slot| Selector::TargetFiltered { slot, filter: R::Creature.and(R::IsAttacking) };
    CardDefinition {
        name: "Command of Unsummoning",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        cast_condition: Some(Predicate::All(vec![
            Predicate::CurrentStepIs(TurnStep::DeclareAttackers),
            Predicate::ValueAtLeast(Value::CreaturesAttackingPlayer(PlayerRef::You), Value::ONE),
        ])),
        effect: Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::Move { what: attacker(0), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
                Effect::Move { what: attacker(1), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            ])),
        },
        ..Default::default()
    }
}

/// Invasion of Vryn // Overloaded Mage-Ring — {3}{U} Siege, defense 4. When it
/// enters, draw three cards, then discard a card. Back: {1}, {T}, sacrifice
/// this artifact: copy target spell you control; you may choose new targets.
pub fn invasion_of_vryn() -> CardDefinition {
    let ring = CardDefinition {
        name: "Overloaded Mage-Ring",
        card_types: vec![CardType::Artifact],
        color_indicator: vec![Color::Blue],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::CopySpellMayChooseTargets {
                what: target_filtered(R::IsSpellOnStack.and(R::ControlledByYou)),
                count: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Vryn",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![BattleSubtype::Siege], ..Default::default() },
        defense: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
        }],
        back_face: Some(Box::new(ring)),
        ..Default::default()
    }
}

/// Fate Transfer — {1}{U/B} instant: move all counters from target creature
/// onto another target creature.
pub fn fate_transfer() -> CardDefinition {
    CardDefinition {
        name: "Fate Transfer",
        cost: cost(&[generic(1), hybrid(Color::Blue, Color::Black)]),
        card_types: vec![CardType::Instant],
        effect: Effect::MoveAllCounters {
            from: target_filtered(R::Creature),
            to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::OtherThanTargetSlot(0)) },
        },
        ..Default::default()
    }
}

/// Silver-Inlaid Dagger — {1} Equipment: +2/+0, a Human an additional +1/+0.
/// Equip {2}.
pub fn silver_inlaid_dagger() -> CardDefinition {
    CardDefinition {
        name: "Silver-Inlaid Dagger",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            conditional: vec![ConditionalEquipBonus {
                host_filter: R::HasCreatureType(CreatureType::Human),
                power: 1,
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Elusive Tormentor // Insidious Mist — {2}{B}{B} 4/4 Vampire Wizard; {1},
/// discard a card: transform it. Back: a 0/1 Elemental with hexproof and
/// indestructible that can't block or be blocked; whenever it attacks and
/// isn't blocked, you may pay {2}{B} to transform it.
pub fn elusive_tormentor() -> CardDefinition {
    let mist = CardDefinition {
        name: "Insidious Mist",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::Hexproof, Keyword::Indestructible, Keyword::CantBlock, Keyword::Unblockable],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {2}{B} to transform it?".into(),
                mana_cost: cost(&[generic(2), b()]),
                body: Box::new(Effect::Transform { what: Selector::This }),
                else_: None,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(mist)),
        ..creature(
            "Elusive Tormentor",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Visions of Duplicity — {2}{U} sorcery: exchange control of two target
/// creatures you don't control. Flashback {8}{U}{U}, {X} less where X is the
/// greatest mana value of a commander you own on the battlefield or in the
/// command zone.
pub fn visions_of_duplicity() -> CardDefinition {
    let theirs = R::Creature.and(R::ControlledByYou.negate());
    CardDefinition {
        name: "Visions of Duplicity",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(8), u(), u()]))],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast this way, where X is the greatest mana value of a commander you own on the battlefield or in the command zone.",
            effect: StaticEffect::SelfFlashbackCostsLess {
                amount: Value::GreatestCommanderManaValueInPlayOrCommandZone(PlayerRef::You),
            },
        }],
        effect: Effect::ExchangeControl {
            a: target_filtered(theirs.clone()),
            b: Selector::TargetFiltered { slot: 1, filter: theirs.and(R::OtherThanTargetSlot(0)) },
        },
        ..Default::default()
    }
}

/// Soulblade Renewer — {4}{G} 2/2 Elf Warrior. Partner with Soulblade
/// Corrupter. When it enters, support 2.
pub fn soulblade_renewer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Soulblade Corrupter".into())],
        triggered_abilities: vec![
            partner_with_search("Soulblade Corrupter"),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: support(2),
            },
        ],
        ..creature("Soulblade Renewer", cost(&[generic(4), g()]), vec![CreatureType::Elf, CreatureType::Warrior], 2, 2)
    }
}

/// Jester's Scepter — {3} artifact. When it enters, exile the top five cards
/// of target player's library face down. {2}, {T}, put a card exiled with it
/// into its owner's graveyard: counter target spell with the same name as that
/// card. The card binned is one that names the spell (a non-matching card
/// never pays for a counter).
pub fn jesters_scepter() -> CardDefinition {
    CardDefinition {
        name: "Jester's Scepter",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileTopOfLibrary {
                who: target_filtered(R::Player),
                amount: Value::Const(5),
                link_to_source: true,
                face_down: true,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::SameNameAsExiledWithSource)) },
                Effect::BinExiledWithSourceNamedLike { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
