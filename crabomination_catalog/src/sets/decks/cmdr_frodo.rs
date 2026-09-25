//! Commander: the cards the **Food and Fellowship** precon (LTC, Frodo,
//! Adventurous Hobbit + Sam, Loyal Attendant) needed beyond what the catalog
//! had. Tests in `tests/recent_b/cmdr_frodo.rs`.
//!
//! Residuals (each also on its card):
//! - **Gollum, Obsessed Stalker** — the drain reaches players this Gollum
//!   has dealt any damage this game, not every creature named Gollum's
//!   combat damage.
//! - **Motivated Pony** — the rider checks for any artifact entering under
//!   your control this turn, not a Food.
//! - **Field-Tested Frying Pan** — the lifegain pump is the Equipment's own
//!   trigger, not an ability the equipped creature has.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, ExileReturnZone, Keyword, MayPlayDuration, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, partner_with_search, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, PlayerStaticTarget, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, w, x};
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn spell(name: &'static str, mana: ManaCost, ty: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![ty], effect, ..Default::default() }
}

fn food() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Food)
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn foods(count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(crate::game::effects::food_token()) }
}

fn a_food() -> Effect {
    foods(Value::ONE)
}

fn white_token(name: &str, types: Vec<CreatureType>, p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn halflings(count: Value) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count,
        definition: Arc::new(white_token("Halfling", vec![CreatureType::Halfling], 1, 1, vec![])),
    }
}

fn gained_three() -> Predicate {
    Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::Const(3) }
}

fn step(s: TurnStep, scope: EventScope) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(s), scope)
}

/// "Target attacking creature gains flying until end of turn."
fn attacker_flies(filter: R) -> Effect {
    Effect::GrantKeyword {
        what: target_filtered(R::Creature.and(R::IsAttacking).and(filter)),
        keyword: Keyword::Flying,
        duration: Duration::EndOfTurn,
    }
}

/// Frodo, Adventurous Hobbit — partner with Sam; vigilance; attacking after
/// gaining 3 life, the Ring tempts you, then a card if Frodo is your
/// Ring-bearer and the Ring has tempted you twice.
pub fn frodo_adventurous_hobbit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Sam, Loyal Attendant".into()), Keyword::Vigilance],
        triggered_abilities: vec![
            partner_with_search("Sam, Loyal Attendant"),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(gained_three()),
                effect: Effect::Seq(vec![
                    Effect::RingTempts { who: PlayerRef::You },
                    Effect::If {
                        cond: Predicate::All(vec![
                            Predicate::EntityMatches { what: Selector::This, filter: R::IsRingBearer },
                            Predicate::RingTemptedAtLeast { who: PlayerRef::You, n: 2 },
                        ]),
                        then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..legendary(creature(
            "Frodo, Adventurous Hobbit",
            cost(&[w(), b()]),
            vec![CreatureType::Halfling, CreatureType::Scout],
            1,
            3,
        ))
    }
}

/// Sam, Loyal Attendant — partner with Frodo; a Food at the beginning of
/// combat on your turn; Foods' activated abilities cost {1} less.
pub fn sam_loyal_attendant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Frodo, Adventurous Hobbit".into())],
        static_abilities: vec![StaticAbility {
            description: "Activated abilities of Foods you control cost {1} less to activate.",
            effect: StaticEffect::MatchingActivatedAbilitiesCostLess { filter: food(), amount: 1 },
        }],
        triggered_abilities: vec![
            partner_with_search("Frodo, Adventurous Hobbit"),
            TriggeredAbility { event: step(TurnStep::BeginCombat, EventScope::YourControl), effect: a_food() },
        ],
        ..legendary(creature(
            "Sam, Loyal Attendant",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Halfling, CreatureType::Peasant],
            2,
            4,
        ))
    }
}

/// Assemble the Entmoot — your creatures have reach; sacrifice it: three
/// tapped X/X Treefolk with reach counters, X = life gained this turn.
pub fn assemble_the_entmoot() -> CardDefinition {
    let gained = || Value::LifeGainedThisTurn(PlayerRef::You);
    let treefolk = TokenDefinition {
        name: "Treefolk".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Treefolk], ..Default::default() },
        dynamic_pt: Some((gained(), gained())),
        tapped: true,
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have reach.",
            effect: StaticEffect::GrantKeyword { applies_to: yours(R::Creature), keyword: Keyword::Reach },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(3), definition: Arc::new(treefolk) },
                Effect::AddKeywordCounter {
                    what: Selector::LastCreatedTokens,
                    keyword: Keyword::Reach,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..spell("Assemble the Entmoot", cost(&[generic(3), g()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Banquet Guests — affinity for Foods; trample; enters with twice X +1/+1
/// counters; {2}, sacrifice a Food: indestructible until end of turn.
pub fn banquet_guests() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        affinity_filter: Some(food()),
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(2))),
        )),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((food(), 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Banquet Guests",
            cost(&[x(), g(), w()]),
            vec![CreatureType::Halfling, CreatureType::Citizen],
            0,
            0,
        )
    }
}

/// Bilbo, Birthday Celebrant — each lifegain is one more; at 111 life,
/// {2}{W}{B}{G}, {T}, exile it: any number of creatures from your library
/// onto the battlefield.
pub fn bilbo_birthday_celebrant() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you would gain life, you gain that much life plus 1 instead.",
            effect: StaticEffect::LifeGainBonus { target: PlayerStaticTarget::Controller, amount: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w(), b(), g()]),
            tap_cost: true,
            exile_self_cost: true,
            condition: Some(Predicate::PlayerLifeAtLeast { who: PlayerRef::You, life: 111 }),
            effect: Effect::SearchAnyNumber {
                who: PlayerRef::You,
                filter: R::Creature,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..legendary(creature(
            "Bilbo, Birthday Celebrant",
            cost(&[w(), b(), g()]),
            vec![CreatureType::Halfling, CreatureType::Rogue],
            2,
            3,
        ))
    }
}

/// Butterbur, Bree Innkeeper — your end step makes a Food if you have none.
pub fn butterbur_bree_innkeeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: step(TurnStep::End, EventScope::YourControl)
                .with_filter(Predicate::Not(Box::new(Predicate::SelectorExists(yours(food()))))),
            effect: a_food(),
        }],
        ..legendary(creature(
            "Butterbur, Bree Innkeeper",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Human, CreatureType::Peasant],
            3,
            3,
        ))
    }
}

/// Call for Unity — revolt: a unity counter at your end step; your creatures
/// get +1/+1 per unity counter.
pub fn call_for_unity() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1 for each unity counter on this enchantment.",
            effect: StaticEffect::PumpPTPerCounterOnSource {
                applies_to: yours(R::Creature),
                kind: CounterType::Unity,
                per_power: 1,
                per_toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: step(TurnStep::End, EventScope::YourControl)
                .with_filter(Predicate::RevoltActive { who: PlayerRef::You }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Unity, amount: Value::ONE },
        }],
        ..spell("Call for Unity", cost(&[generic(3), w(), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Farmer Cotton — entering, X 1/1 Halflings and X Foods.
pub fn farmer_cotton() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![halflings(Value::XFromCost), foods(Value::XFromCost)]))],
        ..legendary(creature(
            "Farmer Cotton",
            cost(&[x(), g(), w()]),
            vec![CreatureType::Halfling, CreatureType::Peasant],
            1,
            1,
        ))
    }
}

/// Feasting Hobbit — devour Food 3; creatures with less power can't block it.
pub fn feasting_hobbit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByPowerLess],
        as_enters_effect: Some(Effect::SacrificeAnyNumber {
            who: PlayerRef::You,
            filter: food(),
            per_each: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            }),
        }),
        ..creature(
            "Feasting Hobbit",
            cost(&[generic(1), g()]),
            vec![CreatureType::Halfling, CreatureType::Citizen],
            2,
            2,
        )
    }
}

/// Field-Tested Frying Pan — entering, a Food and a 1/1 Halfling it attaches
/// to; the equipped creature grows by each lifegain until end of turn.
///
/// ⚠ Residual: the pump is the Equipment's trigger, not the creature's.
pub fn field_tested_frying_pan() -> CardDefinition {
    CardDefinition {
        name: "Field-Tested Frying Pan",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                a_food(),
                halflings(Value::ONE),
                Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    power: Value::TriggerEventAmount,
                    toughness: Value::TriggerEventAmount,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        ..Default::default()
    }
}

/// Gollum, Obsessed Stalker — skulk; your end step drains the opponents Gollum
/// has hit by the life you gained this turn.
///
/// ⚠ Residual: "dealt combat damage this game by a creature named Gollum"
/// reads the players this Gollum has dealt any damage this game.
pub fn gollum_obsessed_stalker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Skulk],
        triggered_abilities: vec![TriggeredAbility {
            event: step(TurnStep::End, EventScope::YourControl),
            effect: Effect::LoseLife {
                who: Selector::DamagedBySourceThisGame,
                amount: Value::LifeGainedThisTurn(PlayerRef::You),
            },
        }],
        ..legendary(creature(
            "Gollum, Obsessed Stalker",
            cost(&[generic(1), b()]),
            vec![CreatureType::Halfling, CreatureType::Horror],
            1,
            1,
        ))
    }
}

/// Gwaihir, Greatest of the Eagles — flying; its attack gives an attacker
/// flying; each end step after you gained 3 life, a 3/3 flying Bird with the
/// same attack trigger.
pub fn gwaihir_greatest_of_the_eagles() -> CardDefinition {
    let bird = TokenDefinition {
        triggered_abilities: vec![on_attack(attacker_flies(R::Any))],
        ..white_token("Bird", vec![CreatureType::Bird], 3, 3, vec![Keyword::Flying])
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            on_attack(attacker_flies(R::Any)),
            TriggeredAbility {
                event: step(TurnStep::End, EventScope::AnyPlayer).with_filter(gained_three()),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(bird) },
            },
        ],
        ..legendary(creature(
            "Gwaihir, Greatest of the Eagles",
            cost(&[generic(4), w()]),
            vec![CreatureType::Bird, CreatureType::Noble],
            5,
            5,
        ))
    }
}

/// Hithlain Rope — can't be sacrificed; a basic land or a card, then it
/// passes to the player on your right.
pub fn hithlain_rope() -> CardDefinition {
    let pass = || Effect::GainControl {
        what: Selector::This,
        to: Some(PlayerRef::PlayerToYourRight),
        duration: Duration::Permanent,
    };
    CardDefinition {
        keywords: vec![Keyword::CantBeSacrificed],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Search {
                        who: PlayerRef::You,
                        filter: R::IsBasicLand,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    },
                    pass(),
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Seq(vec![Effect::Draw { who: Selector::You, amount: Value::ONE }, pass()]),
                ..Default::default()
            },
        ],
        ..spell("Hithlain Rope", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// Landroval, Horizon Witness — flying; two or more attackers give an
/// attacker without flying flying.
pub fn landroval_horizon_witness() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                .with_filter(Predicate::AttackedWithCountAtLeast { who: PlayerRef::You, at_least: 2 }),
            effect: attacker_flies(R::HasKeyword(Keyword::Flying).negate()),
        }],
        ..legendary(creature(
            "Landroval, Horizon Witness",
            cost(&[generic(4), w()]),
            vec![CreatureType::Bird, CreatureType::Noble],
            3,
            4,
        ))
    }
}

/// Lobelia, Defender of Bag End — entering, exiles each opponent's top card
/// face down; {T}, sacrifice an artifact: play one of them free this turn, or
/// drain 2.
pub fn lobelia_defender_of_bag_end() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileTopOfLibrary {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
            link_to_source: true,
            face_down: true,
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::ChooseMode(vec![
                Effect::GrantMayPlay {
                    what: Selector::one_of(Selector::CardExiledWithSource),
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: false,
                    any_color: false,
                },
                Effect::Drain { from: Selector::Player(PlayerRef::EachOpponent), to: Selector::You, amount: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        ..legendary(creature(
            "Lobelia, Defender of Bag End",
            cost(&[generic(2), b()]),
            vec![CreatureType::Halfling, CreatureType::Citizen],
            2,
            2,
        ))
    }
}

/// Merry, Warden of Isengard — partner with Pippin; once a turn, artifacts
/// entering make a 1/1 lifelink Soldier.
pub fn merry_warden_of_isengard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Pippin, Warden of Isengard".into())],
        triggered_abilities: vec![
            partner_with_search("Pippin, Warden of Isengard"),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact })
                    .once_per_turn(),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(white_token(
                        "Soldier",
                        vec![CreatureType::Soldier],
                        1,
                        1,
                        vec![Keyword::Lifelink],
                    )),
                },
            },
        ],
        ..legendary(creature(
            "Merry, Warden of Isengard",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Halfling, CreatureType::Advisor],
            1,
            4,
        ))
    }
}

/// Pippin, Warden of Isengard — partner with Merry; {1}, {T}: a Food; {T},
/// sacrifice four Foods: other creatures +3/+3 and haste (sorcery speed).
pub fn pippin_warden_of_isengard() -> CardDefinition {
    let others = || yours(R::Creature.and(R::OtherThanSource));
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Merry, Warden of Isengard".into())],
        triggered_abilities: vec![partner_with_search("Merry, Warden of Isengard")],
        activated_abilities: vec![
            ActivatedAbility { mana_cost: cost(&[generic(1)]), tap_cost: true, effect: a_food(), ..Default::default() },
            ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                sac_other_filter: Some((food(), 4)),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: others(),
                        power: Value::Const(3),
                        toughness: Value::Const(3),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: others(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ]),
                ..Default::default()
            },
        ],
        ..legendary(creature(
            "Pippin, Warden of Isengard",
            cost(&[b(), g()]),
            vec![CreatureType::Halfling, CreatureType::Advisor],
            2,
            2,
        ))
    }
}

/// Motivated Pony — trample, haste; attacking pumps the attackers +1/+1, and
/// after a Food entered this turn untaps them with +2/+2 more.
///
/// ⚠ Residual: the rider checks for any artifact entering this turn.
pub fn motivated_pony() -> CardDefinition {
    let attackers = || yours(R::Creature.and(R::IsAttacking));
    let pump = |n: i32| Effect::PumpPT {
        what: attackers(),
        power: Value::Const(n),
        toughness: Value::Const(n),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            pump(1),
            Effect::If {
                cond: Predicate::ArtifactEnteredThisTurn { who: PlayerRef::You },
                then: Box::new(Effect::Seq(vec![Effect::Untap { what: attackers(), up_to: None }, pump(2)])),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature("Motivated Pony", cost(&[generic(4), g()]), vec![CreatureType::Horse], 3, 3)
    }
}

/// Of Herbs and Stewed Rabbit — Saga: a +1/+1 counter and a Food; a card and a
/// Food; a Halfling per Food you control.
pub fn of_herbs_and_stewed_rabbit() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 0,
                        filter: R::Creature,
                        effect: Box::new(Effect::AddCounter {
                            what: Selector::Target(0),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        }),
                    },
                    a_food(),
                ]),
            ),
            (2, Effect::Seq(vec![Effect::Draw { who: Selector::You, amount: Value::ONE }, a_food()])),
            (3, halflings(Value::CountOf(Box::new(yours(food()))))),
        ],
        ..spell("Of Herbs and Stewed Rabbit", cost(&[generic(2), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Prize Pig — each lifegain adds that many ribbon counters; at three it
/// sheds them and untaps; {T}: one mana of any color.
pub fn prize_pig() -> CardDefinition {
    let ribbons = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Ribbon };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Ribbon, amount: Value::TriggerEventAmount },
                Effect::If {
                    cond: Predicate::ValueAtLeast(ribbons(), Value::Const(3)),
                    then: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter { what: Selector::This, kind: CounterType::Ribbon, amount: ribbons() },
                        Effect::Untap { what: Selector::This, up_to: None },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        activated_abilities: vec![super::super::tap_add_any_color()],
        ..creature("Prize Pig", cost(&[generic(1), g()]), vec![CreatureType::Boar], 0, 3)
    }
}

/// Rapacious Guest — menace; a Food per combat-damage batch to a player; a
/// +1/+1 counter per Food you sacrifice; leaving, target opponent loses life
/// equal to its power.
pub fn rapacious_guest() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).once_per_batch(),
                effect: a_food(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: food() }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::LoseLife {
                    who: target_filtered(R::OpponentPlayer),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
            },
        ],
        ..creature(
            "Rapacious Guest",
            cost(&[generic(2), b()]),
            vec![CreatureType::Halfling, CreatureType::Citizen],
            2,
            2,
        )
    }
}

/// Revive the Shire — a permanent card from your graveyard to hand, and a
/// Food.
pub fn revive_the_shire() -> CardDefinition {
    spell(
        "Revive the Shire",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::PermanentCard.from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            a_food(),
        ]),
    )
}

/// Rosie Cotton of South Lane — entering, a Food; each token you create puts
/// a +1/+1 counter on another creature of yours.
pub fn rosie_cotton_of_south_lane() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(a_food()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::TokenCreated, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..legendary(creature(
            "Rosie Cotton of South Lane",
            cost(&[generic(2), w()]),
            vec![CreatureType::Halfling, CreatureType::Peasant],
            1,
            1,
        ))
    }
}

/// Savvy Hunter — attacking or blocking makes a Food; sacrifice two Foods:
/// draw a card.
pub fn savvy_hunter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(a_food()),
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: a_food() },
        ],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((food(), 2)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Savvy Hunter",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Shire Shirriff — vigilance; entering, you may sacrifice a token to exile
/// an opponent's creature until it leaves.
pub fn shire_shirriff() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice a token to exile a creature an opponent controls?".into(),
            filter: R::IsToken,
            count: Value::ONE,
            then: Box::new(Effect::Reflexive {
                body: Box::new(Effect::ExileUntilSourceLeaves {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    return_to: ExileReturnZone::Battlefield,
                }),
            }),
            else_: None,
        })],
        ..creature(
            "Shire Shirriff",
            cost(&[generic(1), w()]),
            vec![CreatureType::Halfling, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// The Gaffer — each end step after you gained 3 life, draw a card.
pub fn the_gaffer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: step(TurnStep::End, EventScope::AnyPlayer).with_filter(gained_three()),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..legendary(creature(
            "The Gaffer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Halfling, CreatureType::Peasant],
            2,
            3,
        ))
    }
}

/// Treebeard, Gracious Host — trample, ward {2}; entering, two Foods; each
/// lifegain puts that many +1/+1 counters on a Halfling or Treefolk.
pub fn treebeard_gracious_host() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![
            etb(foods(Value::Const(2))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: target_filtered(
                        R::HasCreatureType(CreatureType::Halfling).or(R::HasCreatureType(CreatureType::Treefolk)),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::TriggerEventAmount,
                },
            },
        ],
        ..legendary(creature(
            "Treebeard, Gracious Host",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Treefolk],
            0,
            5,
        ))
    }
}
