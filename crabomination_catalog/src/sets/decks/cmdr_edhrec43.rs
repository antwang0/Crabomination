//! Commander: the 2026-10-13 rescan's gaps for Tsagan, Raider Warlord and The
//! Balrog, Durin's Bane. Tests in `tests/recent_b/cmdr_edhrec43.rs`.

use crate::card::{
    AdditionalCastCost, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef};
use crate::mana::{b, cost, generic, r, w, ManaCost};
use crabomination_base::tokens::treasure_token;
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn on_attack(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect }
}

fn yours() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn attackers() -> Value {
    Value::count(Selector::EachPermanent(yours().and(R::IsAttacking)))
}

fn treasures(count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(treasure_token()) }
}

// ── Tsagan, Raider Warlord ───────────────────────────────────────────────────

/// Tsagan, Raider Warlord — {R}{W}{B} legendary 1/4 Dinosaur Berserker. Double
/// strike; start your engines! Attacking: your creatures get +1/+0 this turn
/// per creature of yours with first strike or double strike. Max speed —
/// deathtouch, and your other creatures have first strike.
pub fn tsagan_raider_warlord() -> CardDefinition {
    let max_speed = || Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 };
    let strikers = Value::count(Selector::EachPermanent(
        yours().and(R::HasKeyword(Keyword::FirstStrike).or(R::HasKeyword(Keyword::DoubleStrike))),
    ));
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike, Keyword::StartYourEngines],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(yours()),
            power: strikers,
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![
            StaticAbility {
                description: "Max speed — Tsagan has deathtouch.",
                effect: StaticEffect::WhileCondition {
                    condition: max_speed(),
                    inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Deathtouch }),
                },
            },
            StaticAbility {
                description: "Max speed — Other creatures you control have first strike.",
                effect: StaticEffect::WhileCondition {
                    condition: max_speed(),
                    inner: Box::new(StaticEffect::GrantKeyword {
                        applies_to: Selector::EachPermanent(yours().and(R::OtherThanSource)),
                        keyword: Keyword::FirstStrike,
                    }),
                },
            },
        ],
        ..legend(
            "Tsagan, Raider Warlord",
            cost(&[r(), w(), b()]),
            vec![CreatureType::Dinosaur, CreatureType::Berserker],
            1,
            4,
        )
    }
}

/// Kinscaer Sentry — {1}{W} 2/2 Kithkin Soldier, first strike, lifelink.
/// Attacking: you may put a creature card with mana value at most your
/// attacker count from your hand onto the battlefield tapped and attacking.
pub fn kinscaer_sentry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Lifelink],
        triggered_abilities: vec![on_attack(Effect::DeployCreatureFromHandAttacking {
            filter: R::Creature.and(R::ManaValueAtMostYourCount(Box::new(yours().and(R::IsAttacking)))),
            return_to_hand_eot: false,
        })],
        ..creature("Kinscaer Sentry", cost(&[generic(1), w()]), vec![CreatureType::Kithkin, CreatureType::Soldier], 2, 2)
    }
}

/// Smellerbee, Rebel Fighter — {3}{R} legendary 3/3 Human Rebel Ally. First
/// strike; your other creatures have haste. Attacking: you may discard your
/// hand to draw one per attacking creature.
pub fn smellerbee_rebel_fighter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours().and(R::OtherThanSource)),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Discard your hand to draw a card per attacking creature?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
                Effect::Draw { who: Selector::You, amount: attackers() },
            ])),
        })],
        ..legend(
            "Smellerbee, Rebel Fighter",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Ally],
            3,
            3,
        )
    }
}

// ── The Balrog, Durin's Bane ─────────────────────────────────────────────────

/// Lobelia Sackville-Baggins — {2}{B} legendary 2/3 Halfling Citizen, flash,
/// menace. Enters: exile a creature card that went to an opponent's graveyard
/// from the battlefield this turn, then make Treasures equal to its power.
pub fn lobelia_sackville_baggins() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Menace],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    R::Creature.and(R::InOpponentGraveyard).and(R::PutIntoGraveyardFromBattlefieldThisTurn),
                ),
            },
            treasures(Value::PowerOf(Box::new(Selector::Target(0)))),
        ]))],
        ..legend(
            "Lobelia Sackville-Baggins",
            cost(&[generic(2), b()]),
            vec![CreatureType::Halfling, CreatureType::Citizen],
            2,
            3,
        )
    }
}

/// Lash of the Balrog — {B} sorcery; as an additional cost sacrifice a
/// creature or pay {4}. Destroy target creature.
pub fn lash_of_the_balrog() -> CardDefinition {
    CardDefinition {
        name: "Lash of the Balrog",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificeOrPay { filter: R::Creature, pay: 4 }],
        effect: Effect::Destroy { what: target_filtered(R::Creature) },
        ..Default::default()
    }
}

/// Visions of Ruin — {3}{R} sorcery. Each opponent sacrifices an artifact;
/// a Treasure per artifact sacrificed. Flashback {8}{R}{R}, {X} less where X
/// is the greatest mana value of a commander you own on the battlefield or in
/// the command zone (CR 903.3).
pub fn visions_of_ruin() -> CardDefinition {
    CardDefinition {
        name: "Visions of Ruin",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(8), r(), r()]))],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast this way, where X is the greatest mana value of a \
                          commander you own on the battlefield or in the command zone.",
            effect: StaticEffect::SelfFlashbackCostsLess {
                amount: Value::GreatestCommanderManaValueInPlayOrCommandZone(PlayerRef::You),
            },
        }],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Artifact,
            },
            treasures(Value::SacrificedCount),
        ]),
        ..Default::default()
    }
}
