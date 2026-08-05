//! Tarkir: Dragonstorm gaps — the five-color Dragon enchantment, Mardu
//! Siegebreaker's exiled-copy swings, New Way Forward's reflexive prevention,
//! Taigam's flurry-suspend and Ugin.
//! Tests in `tests/classic_sets/tdm2.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, ExileReturnZone, Keyword, LoyaltyAbility,
    MayPlayDuration, PlaneswalkerSubtype, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, flurry, on_attack, on_cast};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate, Selector, ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, u, w};

fn dragons_you_control() -> R {
    R::HasCreatureType(CreatureType::Dragon).and(R::ControlledByYou)
}

/// Call the Spirit Dragons — {W}{U}{B}{R}{G} Enchantment. Your Dragons are
/// indestructible; each upkeep one Dragon of each color grows, and five
/// distinct recipients wins the game.
pub fn call_the_spirit_dragons() -> CardDefinition {
    CardDefinition {
        name: "Call the Spirit Dragons",
        cost: cost(&[w(), u(), b(), r(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Dragons you control have indestructible.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(dragons_you_control()),
                keyword: Keyword::Indestructible,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::CounterOnMatchingOfEachColor {
                filter: dragons_you_control(),
                kind: CounterType::PlusOnePlusOne,
                win_at: 5,
            },
        }],
        ..Default::default()
    }
}

/// Mardu Siegebreaker — {1}{R}{W}{B} 4/4 deathtouch haste. It banishes one of
/// your creatures and swings a temporary copy of it at each opponent.
pub fn mardu_siegebreaker() -> CardDefinition {
    CardDefinition {
        name: "Mardu Siegebreaker",
        cost: cost(&[generic(1), r(), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch, Keyword::Haste],
        triggered_abilities: vec![
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::ExileUntilSourceLeaves {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    },
                    return_to: ExileReturnZone::Battlefield,
                }),
            }),
            on_attack(Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::OpponentCount,
                    source: Selector::CardExiledWithSource,
                    enters_tapped: true,
                    extra_creature_types: Vec::new(),
                    extra_card_types: Vec::new(),
                    override_pt: None,
                    override_colors: None,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: Vec::new(),
                },
                Effect::JoinCombatAttacking { what: Selector::LastCreatedTokens },
                Effect::SacrificeLastCreatedTokensAtNextEndStep,
            ])),
        ],
        ..Default::default()
    }
}

/// New Way Forward — {2}{U}{R}{W} Instant. Shrug off the next hit and send it
/// back, drawing that many cards.
pub fn new_way_forward() -> CardDefinition {
    CardDefinition {
        name: "New Way Forward",
        cost: cost(&[generic(2), u(), r(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PreventNextDamageToYouFromChosenSourceWithRider {
            filter: R::Any,
            rider: Box::new(Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(
                        Selector::TriggerSource,
                    ))),
                    amount: Value::TriggerEventAmount,
                },
                Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
            ])),
        },
        ..Default::default()
    }
}

/// Taigam, Master Opportunist — {1}{U} 2/2 Monk. Your second spell each turn
/// is copied, then suspended with four time counters so you get it twice more.
pub fn taigam_master_opportunist() -> CardDefinition {
    CardDefinition {
        name: "Taigam, Master Opportunist",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![flurry(Effect::Seq(vec![
            Effect::CopySpell { what: Selector::TriggerSource, count: Value::ONE },
            Effect::GrantSuspend { what: Selector::TriggerSource, time_counters: 4 },
        ]))],
        ..Default::default()
    }
}

/// Ugin, Eye of the Storms — {7} loyalty 7. Casting him and every later
/// colorless spell exiles a colored permanent; the ultimate tutors your whole
/// colorless top end into a free-cast pile.
pub fn ugin_eye_of_the_storms() -> CardDefinition {
    let exile_colored = || Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::Not(Box::new(R::Colorless)),
            },
            to: ZoneDest::Exile,
        }),
    };
    CardDefinition {
        name: "Ugin, Eye of the Storms",
        cost: cost(&[generic(7)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Ugin],
            ..Default::default()
        },
        base_loyalty: 7,
        triggered_abilities: vec![
            on_cast(exile_colored()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Colorless,
                    }),
                effect: exile_colored(),
            },
        ],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(3)),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -11,
                effect: Effect::Seq(vec![
                    Effect::SearchAnyNumber {
                        who: PlayerRef::You,
                        filter: R::Colorless.and(R::Not(Box::new(R::Land))),
                        to: ZoneDest::Exile,
                    },
                    Effect::GrantMayPlay {
                        what: Selector::ExiledThisResolution { filter: R::Any },
                        duration: MayPlayDuration::EndOfThisTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: false,
                        any_color: false,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
