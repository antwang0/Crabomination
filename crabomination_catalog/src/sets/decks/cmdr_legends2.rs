//! Most-built commanders missing from the catalog (COMMANDER_BACKLOG §1,
//! regenerated 2026-09-27): Reaper King, Shroofus Sproutsire, Kratos, God of
//! War, Myrel, Shield of Argive, Doran, Besieged by Time, Gargos, Vicious
//! Watcher, Syr Gwyn, Hero of Ashvale, Jodah, Archmage Eternal, Raggadragga,
//! Goreguts Boss, Alexios, Deimos of Kosmos, Narset, Enlightened Exile,
//! Thalia and The Gitrog Monster, Rocco, Cabaretti Caterer, Anti-Venom,
//! Horrifying Healer, Sonic the Hedgehog, Iron Man, Titan of Innovation, and
//! the Father & son partners Kratos, Stoic Father and Atreus, Impulsive Son,
//! Jin Sakai, Ghost of Tsushima (`Predicate::TriggerSourceAttacksItsPlayerAlone`),
//! and the Partner planeswalker commanders Jeska, Thrice Reborn and Tevesh
//! Szat, Doom of Fools.
//! All but Syr Gwyn are built from
//! primitives other cards already use; Syr Gwyn's "Equipment you control have
//! equip Knight {0}" is `StaticEffect::EquipmentYouControlEquipZeroFor`
//! (CR 702.6c, an equip ability restricted to a quality).
//!
//! - **Gargos** — "up to one target creature you don't control" takes the
//!   target when one exists; with none the trigger is removed (CR 603.3d),
//!   the same as choosing zero.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Predicate};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, mono_hybrid, r, u, w};

fn legend(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn trigger_source_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

/// Reaper King — other Scarecrows you control get +1/+1; another Scarecrow
/// of yours entering destroys target permanent.
pub fn reaper_king() -> CardDefinition {
    let scarecrow = || R::HasCreatureType(CreatureType::Scarecrow);
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "Other Scarecrow creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    scarecrow().and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(trigger_source_is(scarecrow().and(R::OtherThanSource))),
            effect: Effect::Destroy { what: target_filtered(R::Permanent) },
        }],
        ..legend(
            "Reaper King",
            cost(&[
                mono_hybrid(2, Color::White),
                mono_hybrid(2, Color::Blue),
                mono_hybrid(2, Color::Black),
                mono_hybrid(2, Color::Red),
                mono_hybrid(2, Color::Green),
            ]),
            vec![CreatureType::Scarecrow],
            6,
            6,
        )
    }
}

/// Shroofus Sproutsire — trample; a Saproling of yours dealing combat damage
/// to a player makes that many 1/1 Saprolings.
pub fn shroofus_sproutsire() -> CardDefinition {
    let saproling = TokenDefinition {
        name: "Saproling".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(trigger_source_is(R::HasCreatureType(CreatureType::Saproling))),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                definition: std::sync::Arc::new(saproling),
            },
        }],
        ..legend("Shroofus Sproutsire", cost(&[generic(2), g()]), vec![CreatureType::Saproling], 1, 1)
    }
}

/// Kratos, God of War — double strike; all creatures have haste; at each
/// player's end step, damage to that player equal to the creatures they
/// control that didn't attack this turn.
pub fn kratos_god_of_war() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        static_abilities: vec![StaticAbility {
            description: "All creatures have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::PermanentCountControlledByMatching(
                    PlayerRef::ActivePlayer,
                    R::Creature.and(R::Not(Box::new(R::AttackedThisTurn))),
                ),
            },
        }],
        ..legend("Kratos, God of War", cost(&[r(), r(), r()]), vec![CreatureType::God, CreatureType::Warrior], 2, 3)
    }
}

/// Myrel, Shield of Argive — during your turn, opponents can't cast spells or
/// activate abilities of artifacts, creatures, or enchantments; attacking
/// makes a 1/1 Soldier artifact creature per Soldier you control.
pub fn myrel_shield_of_argive() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, your opponents can't cast spells or activate abilities of artifacts, creatures, or enchantments.",
            effect: StaticEffect::OpponentsCantActDuringYourTurn,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::PermanentCountControlledByMatching(
                    PlayerRef::You,
                    R::HasCreatureType(CreatureType::Soldier),
                ),
                definition: std::sync::Arc::new(soldier),
            },
        }],
        ..legend(
            "Myrel, Shield of Argive",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Doran, Besieged by Time — creature spells with toughness greater than
/// power cost {1} less; a creature of yours attacking or blocking gets +X/+X,
/// X the difference between its power and toughness.
pub fn doran_besieged_by_time() -> CardDefinition {
    let pump = || {
        let p = || Value::PowerOf(Box::new(Selector::TriggerSource));
        let t = || Value::ToughnessOf(Box::new(Selector::TriggerSource));
        let x = Value::Max(Box::new(Value::Diff(Box::new(p()), Box::new(t()))), Box::new(Value::Diff(Box::new(t()), Box::new(p()))));
        Effect::PumpPT { what: Selector::TriggerSource, power: x.clone(), toughness: x, duration: Duration::EndOfTurn }
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature spell you cast with toughness greater than its power costs {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Creature.and(R::ToughnessGreaterThanPower), amount: 1 },
        }],
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::YourControl), effect: pump() },
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::YourControl), effect: pump() },
        ],
        ..legend(
            "Doran, Besieged by Time",
            cost(&[generic(1), w(), b(), g()]),
            vec![CreatureType::Treefolk, CreatureType::Druid],
            0,
            5,
        )
    }
}

/// Gargos, Vicious Watcher — vigilance; Hydra spells cost {4} less; a
/// creature of yours becoming the target of a spell makes Gargos fight a
/// creature you don't control.
pub fn gargos_vicious_watcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Hydra spells you cast cost {4} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::HasCreatureType(CreatureType::Hydra), amount: 4 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                causer_filter: Some(R::IsSpellOnStack),
                ..EventSpec::new(EventKind::BecameTarget, EventScope::YourCreatureTargeted)
            },
            effect: Effect::Fight {
                attacker: Selector::This,
                defender: target_filtered(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
            },
        }],
        ..legend("Gargos, Vicious Watcher", cost(&[generic(3), g(), g(), g()]), vec![CreatureType::Hydra], 8, 7)
    }
}

/// Syr Gwyn, Hero of Ashvale — vigilance, menace; an equipped creature of
/// yours attacking draws a card and loses 1 life; Equipment you control have
/// equip Knight {0}.
pub fn syr_gwyn_hero_of_ashvale() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Equipment you control have equip Knight {0}.",
            effect: StaticEffect::EquipmentYouControlEquipZeroFor { filter: R::HasCreatureType(CreatureType::Knight) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(trigger_source_is(R::IsEquipped)),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..legend(
            "Syr Gwyn, Hero of Ashvale",
            cost(&[generic(3), r(), w(), b()]),
            vec![CreatureType::Human, CreatureType::Knight],
            5,
            5,
        )
    }
}

/// Jodah, Archmage Eternal — flying; you may pay {W}{U}{B}{R}{G} rather than
/// the mana cost for spells you cast.
pub fn jodah_archmage_eternal() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You may pay {W}{U}{B}{R}{G} rather than pay the mana cost for spells you cast.",
            effect: StaticEffect::FiveColorAlternativeCost,
        }],
        ..legend(
            "Jodah, Archmage Eternal",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            4,
            3,
        )
    }
}

/// Raggadragga, Goreguts Boss — your creatures with a mana ability get +2/+2
/// and untap when they attack; a spell cast with seven or more mana untaps
/// target creature and gives it +7/+7 and trample.
pub fn raggadragga_goreguts_boss() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a mana ability gets +2/+2.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::HasManaAbility)),
                power: 2,
                toughness: 2,
            },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                    .with_filter(trigger_source_is(R::HasManaAbility)),
                effect: Effect::Untap { what: Selector::TriggerSource, up_to: None },
            },
            TriggeredAbility {
                // CR 603.4 — "if at least seven mana was spent to cast it".
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellManaSpentAtLeast(7)),
                effect: Effect::Seq(vec![
                    Effect::Untap { what: target_filtered(R::Creature), up_to: None },
                    Effect::PumpPT {
                        what: Selector::Target(0),
                        power: Value::Const(7),
                        toughness: Value::Const(7),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                ]),
            },
        ],
        ..legend(
            "Raggadragga, Goreguts Boss",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Human, CreatureType::Boar],
            4,
            4,
        )
    }
}

/// Alexios, Deimos of Kosmos — trample; attacks each combat, can't be
/// sacrificed, can't attack its owner; at each player's upkeep that player
/// takes it, untaps it, and gives it a +1/+1 counter and haste.
pub fn alexios_deimos_of_kosmos() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::MustAttack, Keyword::CantBeSacrificed, Keyword::CantAttackOwner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::GainControl { what: Selector::This, to: Some(PlayerRef::ActivePlayer), duration: Duration::Permanent },
                Effect::Untap { what: Selector::This, up_to: None },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            ]),
        }],
        ..legend(
            "Alexios, Deimos of Kosmos",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Berserker],
            4,
            4,
        )
    }
}

/// Narset, Enlightened Exile — your creatures have prowess; attacking exiles
/// a noncreature, nonland card with mana value less than Narset's power from
/// a graveyard and offers a free cast of a copy of it.
pub fn narset_enlightened_exile() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have prowess.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Prowess,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileWithSource {
                    what: target_filtered(
                        R::Noncreature.and(R::Nonland).and(R::InGraveyard).and(R::ManaValueLessThanSourcePower),
                    ),
                },
                Effect::CopyCardAndCastFree { what: Selector::CardExiledWithSource },
            ]),
        }],
        ..legend(
            "Narset, Enlightened Exile",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Monk],
            3,
            4,
        )
    }
}

/// Thalia and The Gitrog Monster — first strike, deathtouch; an additional
/// land each turn; opponents' creatures and nonbasic lands enter tapped;
/// attacking sacrifices a creature or land, then draws a card.
pub fn thalia_and_the_gitrog_monster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Deathtouch],
        static_abilities: vec![
            StaticAbility { description: "You may play an additional land on each of your turns.", effect: StaticEffect::ExtraLandPerTurn },
            StaticAbility {
                description: "Creatures and nonbasic lands your opponents control enter tapped.",
                effect: StaticEffect::EntersTapped {
                    applies_to: Selector::EachPermanent(R::ControlledByOpponent.and(R::Creature.or(R::IsNonbasicLand))),
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature.or(R::Land) },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..legend(
            "Thalia and The Gitrog Monster",
            cost(&[generic(1), w(), b(), g()]),
            vec![CreatureType::Human, CreatureType::Frog, CreatureType::Horror],
            4,
            4,
        )
    }
}

/// Rocco, Cabaretti Caterer — {X}{R}{G}{W}; entering, if cast, may search
/// for a creature card with mana value X or less and put it onto the
/// battlefield.
pub fn rocco_cabaretti_caterer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::If {
            cond: Predicate::SourceWasCast,
            then: Box::new(Effect::MayDo {
                description: "Search for a creature card with mana value X or less?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::ManaValueAtMostXFromCost),
                    to: crate::effect::ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..legend(
            "Rocco, Cabaretti Caterer",
            cost(&[crate::mana::x(), r(), g(), w()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            3,
            1,
        )
    }
}

/// Anti-Venom, Horrifying Healer — entering, if cast, returns target
/// creature card from your graveyard to the battlefield; damage to him
/// becomes that many +1/+1 counters instead.
pub fn anti_venom_horrifying_healer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to Anti-Venom, prevent that damage and put that many +1/+1 counters on him.",
            effect: StaticEffect::ReplaceDamageToSelfWithCounters { kind: CounterType::PlusOnePlusOne },
        }],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::If {
            cond: Predicate::SourceWasCast,
            then: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard).and(R::OwnedByYou)),
                to: crate::effect::ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..legend(
            "Anti-Venom, Horrifying Healer",
            cost(&[w(), w(), w(), w(), w()]),
            vec![CreatureType::Symbiote, CreatureType::Hero],
            5,
            5,
        )
    }
}

/// Sonic the Hedgehog — haste; attacking puts a +1/+1 counter on each of
/// your creatures with flash or haste; one of them being dealt damage makes
/// a tapped Treasure.
pub fn sonic_the_hedgehog() -> CardDefinition {
    let fast = || R::HasKeyword(Keyword::Flash).or(R::HasKeyword(Keyword::Haste));
    let mut treasure = crate::game::effects::treasure_token();
    treasure.tapped = true;
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(fast())),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::YourControl)
                    .with_filter(trigger_source_is(R::Creature.and(fast()))),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: std::sync::Arc::new(treasure) },
            },
        ],
        ..legend(
            "Sonic the Hedgehog",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Hedgehog, CreatureType::Warrior],
            2,
            4,
        )
    }
}

/// Iron Man, Titan of Innovation — flying, haste; attacking makes a
/// Treasure, then you may sacrifice a noncreature artifact to put an artifact
/// card with mana value one greater onto the battlefield tapped.
pub fn iron_man_titan_of_innovation() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: std::sync::Arc::new(crate::game::effects::treasure_token()),
                },
                Effect::MayDo {
                    description: "Sacrifice a noncreature artifact to tutor one a mana value higher?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Artifact.and(R::Noncreature) },
                        Effect::Search {
                            who: PlayerRef::You,
                            filter: R::Artifact.and(R::ManaValueEqualsSacrificedPlus(1)),
                            to: crate::effect::ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                        },
                    ])),
                },
            ]),
        }],
        ..legend(
            "Iron Man, Titan of Innovation",
            cost(&[generic(3), u(), r()]),
            vec![CreatureType::Human, CreatureType::Hero],
            4,
            4,
        )
    }
}

/// Kratos, Stoic Father — you get an experience counter whenever you attack
/// with one or more Gods and whenever a God dies; at your end step, +1/+1
/// counters on target creature equal to your experience. Partner—Father &
/// son.
pub fn kratos_stoic_father() -> CardDefinition {
    let god = || R::HasCreatureType(CreatureType::God);
    CardDefinition {
        keywords: vec![Keyword::PartnerLabel("Father & son".into())],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                    .with_filter(Predicate::AttackedWithCreatureMatching { who: PlayerRef::You, filter: god() }),
                effect: Effect::AddExperience(Value::ONE),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(trigger_source_is(god())),
                effect: Effect::AddExperience(Value::ONE),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ControllerExperience,
                },
            },
        ],
        ..legend(
            "Kratos, Stoic Father",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::God, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Atreus, Impulsive Son — reach; {3}, {T}: draw a card per experience
/// counter you have, then discard a card; 2 damage to each opponent.
/// Partner—Father & son.
pub fn atreus_impulsive_son() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::PartnerLabel("Father & son".into())],
        activated_abilities: vec![crate::card::ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ControllerExperience },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Atreus, Impulsive Son",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::God, CreatureType::Archer],
            2,
            4,
        )
    }
}

/// Jin Sakai, Ghost of Tsushima — combat damage to a player draws a card; a
/// creature of yours attacking a player no other creature is attacking gets
/// the choice of double strike ("Standoff") or can't be blocked ("Ghost").
pub fn jin_sakai_ghost_of_tsushima() -> CardDefinition {
    let this_turn = |keyword| Effect::GrantKeyword { what: Selector::TriggerSource, keyword, duration: Duration::EndOfTurn };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
            TriggeredAbility {
                // CR 603.4 — "if no other creatures are attacking that player".
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                    .with_filter(Predicate::TriggerSourceAttacksItsPlayerAlone),
                effect: Effect::ChooseMode(vec![this_turn(Keyword::DoubleStrike), this_turn(Keyword::Unblockable)]),
            },
        ],
        ..legend(
            "Jin Sakai, Ghost of Tsushima",
            cost(&[generic(1), w(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Samurai],
            2,
            4,
        )
    }
}

fn walker_commander(
    name: &'static str,
    mana: crate::mana::ManaCost,
    subtype: crate::card::PlaneswalkerSubtype,
    loyalty: u32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![subtype], ..Default::default() },
        base_loyalty: loyalty,
        can_be_commander: true,
        ..Default::default()
    }
}

/// Jeska, Thrice Reborn — enters with a loyalty counter per command-zone
/// cast of your commanders (CR 903.8's count); 0: a creature's combat damage
/// to your opponents is tripled until your next turn; −X: X damage to each of
/// up to three targets. Partner; can be your commander.
pub fn jeska_thrice_reborn() -> CardDefinition {
    use crate::card::LoyaltyAbility;
    CardDefinition {
        keywords: vec![Keyword::Partner],
        enters_with_counters: Some((CounterType::Loyalty, Value::CommanderCastsFromCommandZone(PlayerRef::You))),
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::TripleCombatDamageToYourOpponentsUntilYourNextTurn { what: target_filtered(R::Creature) },
                ..Default::default()
            },
            LoyaltyAbility {
                x_cost: true,
                effect: Effect::ApplyToTargets {
                    max_targets: 3,
                    min_targets: 0,
                    filter: R::Creature.or(R::Planeswalker).or(R::Player),
                    effect: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::XFromCost }),
                },
                ..Default::default()
            },
        ],
        ..walker_commander(
            "Jeska, Thrice Reborn",
            cost(&[generic(2), r()]),
            crate::card::PlaneswalkerSubtype::Jeska,
            0,
        )
    }
}

/// Tevesh Szat, Doom of Fools — +2: two 0/1 Thrulls; +1: you may sacrifice
/// another creature or planeswalker to draw two, a third if it was a
/// commander; −10: gain control of all commanders and put every commander in
/// a command zone onto the battlefield. Partner; can be your commander.
pub fn tevesh_szat_doom_of_fools() -> CardDefinition {
    use crate::card::LoyaltyAbility;
    let thrull = std::sync::Arc::new(TokenDefinition {
        name: "Thrull".into(),
        colors: vec![Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thrull], ..Default::default() },
        power: 0,
        toughness: 1,
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Partner],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: thrull },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::MayDo {
                    description: "Sacrifice another creature or planeswalker to draw two?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Sacrifice {
                            who: Selector::You,
                            count: Value::ONE,
                            filter: R::Creature.or(R::Planeswalker).and(R::OtherThanSource),
                        },
                        Effect::If {
                            cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                            then: Box::new(Effect::Seq(vec![
                                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                                Effect::Draw { who: Selector::You, amount: Value::CommandersSacrificedThisResolution },
                            ])),
                            else_: Box::new(Effect::Noop),
                        },
                    ])),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -10,
                effect: Effect::Seq(vec![
                    Effect::GainControl {
                        what: Selector::EachPermanent(R::IsCommander),
                        to: None,
                        duration: Duration::Permanent,
                    },
                    Effect::PutCommandersFromCommandZonesOntoBattlefield,
                ]),
                ..Default::default()
            },
        ],
        ..walker_commander(
            "Tevesh Szat, Doom of Fools",
            cost(&[generic(4), b()]),
            crate::card::PlaneswalkerSubtype::Szat,
            4,
        )
    }
}
