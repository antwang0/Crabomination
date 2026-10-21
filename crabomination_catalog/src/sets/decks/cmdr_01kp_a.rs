//! Commander: the 2026-10-21 (`01KPxZ3H`) single-commander closers. Tests in
//! `tests/recent_b/cmdr_01kp_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{craft, target_any, target_filtered};
use crate::effect::{Duration, Effect, LookExileGrant, ManaPayload, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};
use std::sync::Arc;

/// Squall, Gunblade Duelist — {R}{W}{B} 3/2 legendary Human Mercenary, first
/// strike. As it enters, choose a number. Whenever one or more creatures attack
/// one of your opponents, if any of them has power or toughness equal to the
/// chosen number, Squall deals damage equal to its power to that player
/// (an intervening-if, CR 603.4 — checked again on resolution).
pub fn squall_gunblade_duelist() -> CardDefinition {
    let hit = Predicate::AttackerOfTriggererPtEqualsSourceChosenNumber;
    CardDefinition {
        name: "Squall, Gunblade Duelist",
        cost: cost(&[r(), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Mercenary], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        as_enters_effect: Some(Effect::ChooseNumberForSource { max: 16, pays_life: false, min: 0, own_creature_pt: true }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::OpponentOfYoursAttacked).with_filter(hit.clone()),
            effect: Effect::If {
                cond: hit,
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Duelist's Flame — {X}{R}{R} instant. Until end of turn, target blocked
/// creature you control gets +X/+0 and gains trample and "whenever this
/// creature deals combat damage to a player, look at that many cards from the
/// top of your library; you may exile a nonland card from among them, put the
/// rest on the bottom in a random order, and cast the exiled card free".
pub fn duelists_flame() -> CardDefinition {
    let rider = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::LookTopExileOneMayPlay {
            count: Value::TriggerEventAmount,
            who: PlayerRef::You,
            grant: LookExileGrant::CastFreeNonland,
        },
    };
    CardDefinition {
        name: "Duelist's Flame",
        cost: cost(&[x(), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::IsBlocked)),
                power: Value::XFromCost,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            Effect::GrantTriggeredAbility { what: Selector::Target(0), trigger: Box::new(rider), duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Captain Rex Nebula — {1}{R}{W} 2/2 legendary Human Pilot Employee. At the
/// beginning of combat on your turn, until end of turn target nonland
/// permanent you control becomes a Vehicle artifact (losing its other card
/// types and subtypes, keeping supertypes) with base P/T each its mana value,
/// crew 2 and Crash Land — "whenever this Vehicle deals damage, roll a d6; if
/// the result equals its mana value, sacrifice it, then it deals that much
/// damage to any target".
pub fn captain_rex_nebula() -> CardDefinition {
    let mv = || Value::ManaValueOf(Box::new(Selector::Target(0)));
    let crash_land = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
        // One arm over every result: `LastDieRoll` is the face only inside an
        // arm (CR 706.4), and a modified result may be 0 or above 6.
        effect: Effect::RollDie {
            sides: 6,
            count: Value::ONE,
            modifier: Value::ZERO,
            reroll_at_most: 0,
            ignore_lowest: 0,
            results: vec![(
                0,
                u8::MAX,
                Effect::If {
                    cond: Predicate::ValueEquals(Value::LastDieRoll, Value::ManaValueOf(Box::new(Selector::This))),
                    then: Box::new(Effect::Seq(vec![
                        Effect::SacrificePermanent { what: Selector::This },
                        Effect::DealDamage { to: target_any(), amount: Value::LastDieRoll },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            )],
            on_doubles: None,
        },
    };
    CardDefinition {
        name: "Captain Rex Nebula",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot, CreatureType::Employee],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::SetCardTypesFor {
                    what: target_filtered(R::Permanent.and(R::ControlledByYou).and(R::Land.negate())),
                    card_types: vec![CardType::Artifact],
                    duration: Duration::EndOfTurn,
                },
                Effect::BecomeArtifactSubtypesFor {
                    what: Selector::Target(0),
                    subtypes: vec![ArtifactSubtype::Vehicle],
                    duration: Duration::EndOfTurn,
                },
                Effect::SetBasePT { what: Selector::Target(0), power: mv(), toughness: mv(), duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Crew(2), duration: Duration::EndOfTurn },
                Effect::GrantTriggeredAbility {
                    what: Selector::Target(0),
                    trigger: Box::new(crash_land),
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Mysterious Limousine — {3}{W}{W} 4/4 artifact Vehicle, crew 2. Whenever it
/// enters or attacks, exile up to one other target creature until it leaves
/// the battlefield (CR 610.3); if a creature is exiled this way, each other
/// card exiled with it returns to the battlefield under its owner's control.
pub fn mysterious_limousine() -> CardDefinition {
    let swap = || {
        Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::ExileUntilSourceLeaves {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    return_to: ExileReturnZone::Battlefield,
                }),
            },
            Effect::ReturnOtherLinkedExiles,
        ])
    };
    CardDefinition {
        name: "Mysterious Limousine",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource), effect: swap() },
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: swap() },
        ],
        ..Default::default()
    }
}

fn etb(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource), effect }
}

/// Tetzin, Gnome Champion // The Golden-Gear Colossus — {U}{R}{W} 2/2
/// legendary artifact creature Gnome. Whenever it or another double-faced
/// artifact you control enters, mill three and you may put an artifact card
/// milled this way into your hand. Craft with six artifacts {4}. The Colossus:
/// 6/6 vigilance, trample; whenever it enters or attacks, transform up to one
/// other target double-faced artifact you control and create two 1/1 Gnome
/// artifact creature tokens.
pub fn tetzin_gnome_champion() -> CardDefinition {
    let gnome = Arc::new(TokenDefinition {
        name: "Gnome".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        ..Default::default()
    });
    let gears = || {
        Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Transform {
                    what: target_filtered(
                        R::Artifact.and(R::ControlledByYou).and(R::HasBackFace).and(R::OtherThanSource),
                    ),
                }),
            },
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: gnome.clone() },
        ])
    };
    let colossus = CardDefinition {
        name: "The Golden-Gear Colossus",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        color_indicator: vec![Color::White, Color::Blue, Color::Red],
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        triggered_abilities: vec![
            etb(gears()),
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: gears() },
        ],
        ..Default::default()
    };
    CardDefinition {
        name: "Tetzin, Gnome Champion",
        cost: cost(&[u(), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.and(R::HasBackFace) },
            ),
            effect: Effect::MillThenToHand { amount: Value::Const(3), filter: R::Artifact, otherwise: None },
        }],
        activated_abilities: vec![craft(cost(&[generic(4)]), R::Artifact, 6)],
        back_face: Some(Box::new(colossus)),
        ..Default::default()
    }
}

/// Chalice of Life // Chalice of Death — {3} artifact. {T}: you gain 1 life;
/// then if you have at least 10 more than your starting life total, transform
/// it. Chalice of Death: {T}: target player loses 5 life.
pub fn chalice_of_life() -> CardDefinition {
    let death = CardDefinition {
        name: "Chalice of Death",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LoseLife { who: target_filtered(R::Player), amount: Value::Const(5) },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Chalice of Life",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::LifeOf(PlayerRef::You),
                        Value::Sum(vec![Value::StartingLifeTotal, Value::Const(10)]),
                    ),
                    then: Box::new(Effect::Transform { what: Selector::This }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        back_face: Some(Box::new(death)),
        ..Default::default()
    }
}

/// Sunbird Standard // Sunbird Effigy — {3} artifact. {T}: add one mana of any
/// color. Craft with one or more {5}. Sunbird Effigy: */* flying, vigilance,
/// haste Bird Construct, P/T the number of colors among the exiled cards used
/// to craft it; {T}: for each of those colors, add one mana of it.
pub fn sunbird_standard() -> CardDefinition {
    let effigy = CardDefinition {
        name: "Sunbird Effigy",
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird, CreatureType::Construct], ..Default::default() },
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Haste],
        dynamic_pt: Some(DynamicPt::ExiledWithSourceColorCount),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OneOfEachColorAmongExiledWithSource },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Sunbird Standard",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            craft(cost(&[generic(5)]), R::Any, 0),
        ],
        back_face: Some(Box::new(effigy)),
        ..Default::default()
    }
}

/// Unstable Glyphbridge // Sandswirl Wanderglyph — {3}{W}{W} artifact. When it
/// enters, if you cast it, for each player you choose a creature with power 2
/// or less they control; destroy every other creature. Craft with artifact
/// {3}{W}{W}. Sandswirl Wanderglyph: 5/3 flying Golem; whenever an opponent
/// casts a spell during their turn, they can't attack you or planeswalkers you
/// control this turn; each opponent who attacked you or a planeswalker you
/// control this turn can't cast spells.
pub fn unstable_glyphbridge() -> CardDefinition {
    let wanderglyph = CardDefinition {
        name: "Sandswirl Wanderglyph",
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        color_indicator: vec![Color::White],
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::IsTurnOf(PlayerRef::Triggerer)),
            effect: Effect::CantAttackPlayerOrWalkersThisTurn { who: PlayerRef::Triggerer, defender: PlayerRef::You },
        }],
        static_abilities: vec![StaticAbility {
            description: "Each opponent who attacked you or a planeswalker you control this turn can't cast spells.",
            effect: StaticEffect::OpponentsWhoAttackedYouCantCast,
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Unstable Glyphbridge",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(Predicate::SourceWasCast),
            effect: Effect::ChooseOnePerPlayerDestroyRest { keep: R::Creature.and(R::PowerAtMost(2)), of: R::Creature },
        }],
        activated_abilities: vec![craft(cost(&[generic(3), w(), w()]), R::Artifact, 1)],
        back_face: Some(Box::new(wanderglyph)),
        ..Default::default()
    }
}

fn spirit() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    })
}

/// Teachings of the Kirin // Kirin-Touched Orochi — {1}{G} Saga. I: mill
/// three, create a 1/1 colorless Spirit. II: a +1/+1 counter on target
/// creature you control. III: exile it, then return it transformed. The
/// Orochi: 1/1 green enchantment creature Snake Monk; whenever it attacks,
/// choose one — exile target creature card from a graveyard, and when you do
/// (CR 603.12) create a 1/1 Spirit; or exile target noncreature card from a
/// graveyard, and when you do put a +1/+1 counter on target creature you
/// control.
pub fn teachings_of_the_kirin() -> CardDefinition {
    let exiled_then = |filter: R, then: Effect| {
        Effect::Seq(vec![
            Effect::Move { what: target_filtered(filter.from_any_graveyard()), to: ZoneDest::Exile },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::LastMoved),
                then: Box::new(Effect::ReflexiveTrigger { body: Box::new(then) }),
                else_: Box::new(Effect::Noop),
            },
        ])
    };
    let counter = || Effect::AddCounter {
        what: target_filtered(R::Creature.and(R::ControlledByYou)),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    let orochi = CardDefinition {
        name: "Kirin-Touched Orochi",
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake, CreatureType::Monk], ..Default::default() },
        color_indicator: vec![Color::Green],
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                exiled_then(
                    R::Creature,
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: spirit() },
                ),
                exiled_then(R::Creature.negate(), counter()),
            ]),
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Teachings of the Kirin",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::Mill { who: Selector::You, amount: Value::Const(3) },
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: spirit() },
                ]),
            ),
            (2, counter()),
            (3, Effect::ExileSelfReturnTransformed),
        ],
        back_face: Some(Box::new(orochi)),
        ..Default::default()
    }
}

/// Storyweave — {2}{G} instant. Choose one — two +1/+1 counters on target
/// creature you control; or two lore counters on target Saga you control
/// (each chapter it reaches triggers, CR 714.2c), and the next time one or
/// more enchantment creatures you control enter this turn, each enters with
/// two additional +1/+1 counters.
pub fn storyweave() -> CardDefinition {
    CardDefinition {
        name: "Storyweave",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga).and(R::ControlledByYou)),
                    kind: CounterType::Lore,
                    amount: Value::Const(2),
                },
                Effect::NextEnchantmentCreaturesEnterWithCountersThisTurn { amount: 2 },
            ]),
        ]),
        ..Default::default()
    }
}
