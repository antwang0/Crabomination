//! Commander: the cards the **Mutant Menace** precon (PIP, The Wise Mothman)
//! needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_mothman.rs`.
//!
//! Residuals (each also on its card):
//! - **The Wise Mothman** — the "up to X target creatures" are your own
//!   greatest-power creatures, chosen on resolution rather than targeted.
//! - **Rampaging Yao Guai** — the artifacts and enchantments are chosen on
//!   resolution rather than targeted.
//! - **Struggle for Project Purity** — Brotherhood draws you one card per
//!   opponent, not per card they actually drew.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EnterMode, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, evolve, monstrosity, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, u, x, Color, ManaCost};

use super::super::tap_add_colorless;

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: chapters,
        ..Default::default()
    }
}

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn rad(who: PlayerRef, n: i32) -> Effect {
    Effect::AddRadCounters { who: Selector::Player(who), amount: Value::Const(n) }
}

fn plus(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn may(description: &str, body: Effect) -> Effect {
    Effect::MayDo { description: description.into(), body: Box::new(body) }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

fn on_combat_damage(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

/// The player a combat-damage trigger's damage went to.
fn damaged() -> PlayerRef {
    PlayerRef::Triggerer
}

fn zombie_or_mutant() -> R {
    R::HasCreatureType(CreatureType::Zombie).or(R::HasCreatureType(CreatureType::Mutant))
}

fn zombie_mutant() -> TokenDefinition {
    TokenDefinition {
        name: "Zombie Mutant".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie, CreatureType::Mutant], ..Default::default() },
        ..Default::default()
    }
}

fn make(def: TokenDefinition, count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(def) }
}

fn proliferate_twice() -> Effect {
    Effect::Seq(vec![Effect::Proliferate, Effect::Proliferate])
}

/// "Whenever one or more nonland cards are milled" — one fire per batch whose
/// amount is how many nonland cards were milled.
fn nonland_milled() -> EventSpec {
    let mut spec = EventSpec::new(EventKind::Milled, EventScope::AnyPlayer)
        .with_filter(trigger_is(R::Nonland))
        .once_per_batch();
    spec.batch_counts_subjects = true;
    spec
}

fn milled_this_turn(filter: R) -> R {
    filter.and(R::InGraveyard).and(R::PutIntoGraveyardFromLibraryThisTurn)
}

// ── Commander ───────────────────────────────────────────────────────────────

/// The Wise Mothman — {1}{B}{G}{U} Legendary Creature — Insect Mutant 3/3.
/// Flying. Whenever it enters or attacks, each player gets a rad counter.
/// Whenever one or more nonland cards are milled, put a +1/+1 counter on each
/// of up to X target creatures, where X is the number of nonland cards milled
/// this way.
///
/// ⚠ Residual: the X creatures are your greatest-power ones, chosen on
/// resolution rather than targeted.
pub fn the_wise_mothman() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(rad(PlayerRef::EachPlayer, 1)),
            on_attack(rad(PlayerRef::EachPlayer, 1)),
            TriggeredAbility {
                event: nonland_milled(),
                effect: plus(
                    Selector::TakeGreatestPower {
                        inner: Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))),
                        count: Box::new(Value::TriggerEventAmount),
                    },
                    Value::ONE,
                ),
            },
        ],
        ..legend(
            "The Wise Mothman",
            cost(&[generic(1), b(), g(), u()]),
            vec![CreatureType::Insect, CreatureType::Mutant],
            3,
            3,
        )
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Agent Frank Horrigan — {5}{B}{G} Legendary Creature — Mutant Warrior 8/6.
/// Trample. Indestructible as long as it attacked this turn. Whenever it
/// enters or attacks, proliferate twice.
pub fn agent_frank_horrigan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Indestructible as long as it attacked this turn.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Indestructible,
                condition: Predicate::SourceAttackedThisTurn,
            },
        }],
        triggered_abilities: vec![etb(proliferate_twice()), on_attack(proliferate_twice())],
        ..legend(
            "Agent Frank Horrigan",
            cost(&[generic(5), b(), g()]),
            vec![CreatureType::Mutant, CreatureType::Warrior],
            8,
            6,
        )
    }
}

/// Alpha Deathclaw — {4}{B}{G} Creature — Lizard Mutant 6/6. Menace, trample.
/// When it enters or becomes monstrous, destroy target permanent.
/// {5}{B}{G}: Monstrosity 4.
pub fn alpha_deathclaw() -> CardDefinition {
    let destroy = || Effect::Destroy { what: target_filtered(R::Permanent) };
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Trample],
        triggered_abilities: vec![
            etb(destroy()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameMonstrous, EventScope::SelfSource),
                effect: destroy(),
            },
        ],
        activated_abilities: vec![monstrosity(cost(&[generic(5), b(), g()]), 4)],
        ..creature(
            "Alpha Deathclaw",
            cost(&[generic(4), b(), g()]),
            vec![CreatureType::Lizard, CreatureType::Mutant],
            6,
            6,
        )
    }
}

/// Bloatfly Swarm — {3}{B} Creature — Insect Mutant 0/0. Flying. Enters with
/// five +1/+1 counters. If damage would be dealt to it while it has a +1/+1
/// counter on it, prevent that damage, remove that many +1/+1 counters, then
/// give each player a rad counter for each counter removed this way.
pub fn bloatfly_swarm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(5))),
        static_abilities: vec![StaticAbility {
            description: "Damage to it removes +1/+1 counters instead, giving each player rad counters.",
            effect: StaticEffect::PreventDamageToSelfWhileCountersForRad { counter: CounterType::PlusOnePlusOne },
        }],
        ..creature("Bloatfly Swarm", cost(&[generic(3), b()]), vec![CreatureType::Insect, CreatureType::Mutant], 0, 0)
    }
}

/// Cathedral Acolyte — {1}{G} Creature — Human Cleric 1/2. Each creature you
/// control with a counter on it has ward {1}. {T}: Put a +1/+1 counter on
/// target creature that entered this turn.
pub fn cathedral_acolyte() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a counter on it has ward {1}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::WithAnyCounter)),
                keyword: Keyword::Ward(WardCost::generic(1)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: plus(target_filtered(R::Creature.and(R::EnteredThisTurn)), Value::ONE),
            ..Default::default()
        }],
        ..creature("Cathedral Acolyte", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 2)
    }
}

/// Hancock, Ghoulish Mayor — {2}{B} Legendary Creature — Zombie Mutant
/// Advisor 2/1. Each other Zombie or Mutant you control gets +X/+X, where X
/// is the number of counters on Hancock. Undying.
pub fn hancock_ghoulish_mayor() -> CardDefinition {
    let x = || Value::TotalCountersOn { what: Box::new(Selector::This) };
    CardDefinition {
        keywords: vec![Keyword::Undying],
        static_abilities: vec![StaticAbility {
            description: "Each other Zombie or Mutant you control gets +X/+X (X: counters on Hancock).",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource).and(zombie_or_mutant()),
                ),
                power: x(),
                toughness: x(),
            },
        }],
        ..legend(
            "Hancock, Ghoulish Mayor",
            cost(&[generic(2), b()]),
            vec![CreatureType::Zombie, CreatureType::Mutant, CreatureType::Advisor],
            2,
            1,
        )
    }
}

/// Harold and Bob, First Numens — {2}{G} Legendary Creature — Treefolk Mutant
/// 3/3. Reach, vigilance. When it dies, if it was a creature, return it to the
/// battlefield. It's an Aura enchantment with enchant Forest you control and
/// "Enchanted Forest has '{T}: Add three mana of any one color. You get two
/// rad counters.'" It loses all other abilities.
pub fn harold_and_bob_first_numens() -> CardDefinition {
    let forest = || R::HasLandType(LandType::Forest).and(R::ControlledByYou);
    let aura = CardDefinition {
        name: "Harold and Bob, First Numens",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(forest()) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted Forest has '{T}: Add three mana of any one color. You get two rad counters.'",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::Seq(vec![
                        Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(3)) },
                        rad(PlayerRef::You, 2),
                    ]),
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Vigilance],
        back_face: Some(Box::new(aura)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::ReturnSelfTransformedAttachedTo {
                host: Selector::take(Selector::EachPermanent(forest()), Value::ONE),
            },
        }],
        ..legend(
            "Harold and Bob, First Numens",
            cost(&[generic(2), g()]),
            vec![CreatureType::Treefolk, CreatureType::Mutant],
            3,
            3,
        )
    }
}

/// Infesting Radroach — {2}{B} Creature — Insect Mutant 2/2. Flying. Can't
/// block. Whenever it deals combat damage to a player, they get that many rad
/// counters. Whenever an opponent mills a nonland card, if it's in your
/// graveyard, you may return it to your hand.
pub fn infesting_radroach() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CantBlock],
        triggered_abilities: vec![
            on_combat_damage(Effect::AddRadCounters {
                who: Selector::Player(damaged()),
                amount: Value::TriggerEventAmount,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Milled, EventScope::FromYourGraveyard)
                    .with_filter(trigger_is(R::Nonland.and(R::Not(Box::new(R::OwnedByYou)))))
                    .once_per_batch(),
                effect: may("Return Infesting Radroach to your hand?", Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        ],
        ..creature("Infesting Radroach", cost(&[generic(2), b()]), vec![CreatureType::Insect, CreatureType::Mutant], 2, 2)
    }
}

/// Jason Bright, Glowing Prophet — {2}{U} Legendary Creature — Zombie Mutant
/// Advisor 2/3. Whenever a Zombie or Mutant you control dies, if its power was
/// different from its base power, draw a card. Come Fly With Me — {2},
/// Sacrifice a creature: Put a +1/+1 counter on target creature you control.
/// It gains flying until end of turn.
pub fn jason_bright_glowing_prophet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(trigger_is(zombie_or_mutant().and(R::PowerDifferentFromBasePower))),
            effect: draw(Value::ONE),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Creature, 1)),
            sac_other_may_be_source: true,
            effect: Effect::Seq(vec![
                plus(target_filtered(R::Creature.and(R::ControlledByYou)), Value::ONE),
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Jason Bright, Glowing Prophet",
            cost(&[generic(2), u()]),
            vec![CreatureType::Zombie, CreatureType::Mutant, CreatureType::Advisor],
            2,
            3,
        )
    }
}

/// Lily Bowen, Raging Grandma — {3}{G} Legendary Creature — Mutant Warrior
/// 0/0. Vigilance. Enters with two +1/+1 counters. At the beginning of your
/// upkeep, double the +1/+1 counters on it if its power is 16 or less.
/// Otherwise, remove all but one, then gain 1 life for each removed.
pub fn lily_bowen_raging_grandma() -> CardDefinition {
    let counters = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne };
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(Value::Const(16), Value::PowerOf(Box::new(Selector::This))),
                then: Box::new(Effect::DoubleCountersOnEach {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                }),
                else_: Box::new(Effect::WithX {
                    x: Value::Diff(Box::new(counters()), Box::new(Value::ONE)),
                    body: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter {
                            what: Selector::This,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::XFromCost,
                        },
                        Effect::GainLife { who: Selector::You, amount: Value::XFromCost },
                    ])),
                }),
            },
        }],
        ..legend(
            "Lily Bowen, Raging Grandma",
            cost(&[generic(3), g()]),
            vec![CreatureType::Mutant, CreatureType::Warrior],
            0,
            0,
        )
    }
}

/// Lumbering Megasloth — {10}{G}{G} Creature — Sloth Mutant 8/8. Costs {1}
/// less for each counter among players and permanents. Trample. Enters
/// tapped.
pub fn lumbering_megasloth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        self_cost_reduction_per: Some((
            Value::Sum(vec![
                Value::TotalCountersOn { what: Box::new(Selector::EachPermanent(R::Any)) },
                Value::CountersAmongPlayers,
            ]),
            1,
        )),
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        ..creature(
            "Lumbering Megasloth",
            cost(&[generic(10), g(), g()]),
            vec![CreatureType::Sloth, CreatureType::Mutant],
            8,
            8,
        )
    }
}

/// Marcus, Mutant Mayor — {3}{G}{U} Legendary Creature — Mutant Advisor 4/4.
/// Vigilance, trample. Whenever a creature you control deals combat damage to
/// a player, draw a card if it has a +1/+1 counter on it; if it doesn't, put
/// a +1/+1 counter on it.
pub fn marcus_mutant_mayor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::If {
                cond: trigger_is(R::WithCounter(CounterType::PlusOnePlusOne)),
                then: Box::new(draw(Value::ONE)),
                else_: Box::new(plus(Selector::TriggerSource, Value::ONE)),
            },
        }],
        ..legend(
            "Marcus, Mutant Mayor",
            cost(&[generic(3), g(), u()]),
            vec![CreatureType::Mutant, CreatureType::Advisor],
            4,
            4,
        )
    }
}

/// Nightkin Ambusher — {2}{U}{B} Creature — Mutant Warrior 4/4. Ward {2}.
/// When it enters, target player gets four rad counters. It can't be blocked
/// as long as defending player has a rad counter.
pub fn nightkin_ambusher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![etb(Effect::TargetPlayerThen {
            filter: R::Player,
            then: Box::new(rad(PlayerRef::Target(0), 4)),
        })],
        static_abilities: vec![StaticAbility {
            description: "Can't be blocked as long as defending player has a rad counter.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Unblockable,
                condition: Predicate::ValueAtLeast(Value::RadCountersAmong(PlayerRef::DefendingPlayer), Value::ONE),
            },
        }],
        ..creature(
            "Nightkin Ambusher",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Mutant, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Piper Wright, Publick Reporter — {1}{U} Legendary Creature — Human
/// Detective 1/2. Whenever it deals combat damage to a player, investigate
/// that many times. Whenever you sacrifice a Clue, put a +1/+1 counter on
/// target creature you control.
pub fn piper_wright_publick_reporter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_combat_damage(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                definition: Arc::new(crabomination_base::tokens::clue_token()),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(trigger_is(R::HasArtifactSubtype(ArtifactSubtype::Clue))),
                effect: plus(target_filtered(R::Creature.and(R::ControlledByYou)), Value::ONE),
            },
        ],
        ..legend(
            "Piper Wright, Publick Reporter",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Detective],
            1,
            2,
        )
    }
}

/// Rampaging Yao Guai — {X}{G}{G}{G} Creature — Bear Mutant 2/2. Vigilance,
/// trample. Enters with X +1/+1 counters. When it enters, destroy any number
/// of target artifacts and/or enchantments with total mana value X or less.
///
/// ⚠ Residual: the permanents are chosen on resolution, not targeted.
pub fn rampaging_yao_guai() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![etb(Effect::DestroyWithinTotalManaValue {
            filter: R::HasCardType(CardType::Artifact).or(R::HasCardType(CardType::Enchantment)),
            cap: Value::XFromCost,
        })],
        ..creature(
            "Rampaging Yao Guai",
            cost(&[x(), g(), g(), g()]),
            vec![CreatureType::Bear, CreatureType::Mutant],
            2,
            2,
        )
    }
}

/// Raul, Trouble Shooter — {1}{U}{B} Legendary Creature — Zombie Mutant Rogue
/// 1/4. Once during each of your turns, you may cast a spell from among cards
/// in your graveyard that were milled this turn. {T}: Each player mills a
/// card.
pub fn raul_trouble_shooter() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Once during each of your turns, you may cast a spell from among cards in your graveyard milled this turn.",
            effect: StaticEffect::GraveyardCastOncePerTurn {
                mv_at_most_counters: None,
                filter: R::Nonland.and(R::PutIntoGraveyardFromLibraryThisTurn),
                exile_after: false,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
            ..Default::default()
        }],
        ..legend(
            "Raul, Trouble Shooter",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Zombie, CreatureType::Mutant, CreatureType::Rogue],
            1,
            4,
        )
    }
}

/// Screeching Scorchbeast — {4}{B}{B} Creature — Bat Mutant 5/5. Flying,
/// menace. Whenever it attacks, each player gets two rad counters. Whenever
/// one or more nonland cards are milled, you may create that many 2/2 black
/// Zombie Mutant tokens. Do this only once each turn.
pub fn screeching_scorchbeast() -> CardDefinition {
    let mut spawn = TriggeredAbility {
        event: nonland_milled(),
        effect: may("Create that many Zombie Mutants?", make(zombie_mutant(), Value::TriggerEventAmount)),
    };
    spawn.event.once_per_turn = true;
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Menace],
        triggered_abilities: vec![on_attack(rad(PlayerRef::EachPlayer, 2)), spawn],
        ..creature(
            "Screeching Scorchbeast",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Bat, CreatureType::Mutant],
            5,
            5,
        )
    }
}

/// Strong, the Brutish Thespian — {4}{G}{G} Legendary Creature — Mutant
/// Berserker 7/7. Ward {2}. Enrage — Whenever Strong is dealt damage, you get
/// three rad counters and put three +1/+1 counters on Strong. You gain life
/// rather than lose life from radiation.
pub fn strong_the_brutish_thespian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        static_abilities: vec![StaticAbility {
            description: "You gain life rather than lose life from radiation.",
            effect: StaticEffect::GainLifeFromRadiation,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::Seq(vec![rad(PlayerRef::You, 3), plus(Selector::This, Value::Const(3))]),
        }],
        ..legend(
            "Strong, the Brutish Thespian",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Mutant, CreatureType::Berserker],
            7,
            7,
        )
    }
}

/// Tato Farmer — {2}{G} Creature — Zombie Mutant Peasant 1/4. Landfall —
/// whenever a land you control enters, you may get two rad counters. {T}: Put
/// target land card in a graveyard that was milled this turn onto the
/// battlefield tapped under your control.
pub fn tato_farmer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(trigger_is(R::Land)),
            effect: may("Get two rad counters?", rad(PlayerRef::You, 2)),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(milled_this_turn(R::Land)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature(
            "Tato Farmer",
            cost(&[generic(2), g()]),
            vec![CreatureType::Zombie, CreatureType::Mutant, CreatureType::Peasant],
            1,
            4,
        )
    }
}

/// The Master, Transcendent — {1}{B}{G}{U} Legendary Artifact Creature —
/// Mutant 2/4. When it enters, target player gets two rad counters. {T}: Put
/// target creature card in a graveyard that was milled this turn onto the
/// battlefield under your control. It's a green Mutant with base power and
/// toughness 3/3.
pub fn the_master_transcendent() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::TargetPlayerThen {
            filter: R::Player,
            then: Box::new(rad(PlayerRef::Target(0), 2)),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(milled_this_turn(R::Creature)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::BecomeColor {
                    what: Selector::LastMoved,
                    colors: vec![Color::Green],
                    duration: Duration::Permanent,
                    additive: false,
                },
                Effect::BecomeCreatureType {
                    what: Selector::LastMoved,
                    creature_types: vec![CreatureType::Mutant],
                    duration: Duration::Permanent,
                },
                Effect::SetBasePT {
                    what: Selector::LastMoved,
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..legend("The Master, Transcendent", cost(&[generic(1), b(), g(), u()]), vec![CreatureType::Mutant], 2, 4)
    }
}

/// Vexing Radgull — {1}{U} Creature — Bird Mutant 1/2. Flying. Whenever it
/// deals combat damage to a player, that player gets two rad counters if they
/// don't have any. Otherwise, proliferate.
pub fn vexing_radgull() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_combat_damage(Effect::If {
            cond: Predicate::ValueAtLeast(Value::RadCountersAmong(damaged()), Value::ONE),
            then: Box::new(Effect::Proliferate),
            else_: Box::new(rad(damaged(), 2)),
        })],
        ..creature("Vexing Radgull", cost(&[generic(1), u()]), vec![CreatureType::Bird, CreatureType::Mutant], 1, 2)
    }
}

/// Watchful Radstag — {2}{G} Creature — Elk Mutant 2/2. Evolve. Whenever it
/// evolves, create a token that's a copy of it.
pub fn watchful_radstag() -> CardDefinition {
    let mut copies = evolve();
    copies.effect = crate::effect::shortcut::token_copy_of(PlayerRef::You, Value::ONE, Selector::This);
    CardDefinition {
        triggered_abilities: vec![evolve(), copies],
        ..creature("Watchful Radstag", cost(&[generic(2), g()]), vec![CreatureType::Elk, CreatureType::Mutant], 2, 2)
    }
}

/// Young Deathclaws — {2}{B}{G} Creature — Lizard Mutant 4/2. Menace. Each
/// creature card in your graveyard has scavenge; the scavenge cost is its
/// mana cost.
pub fn young_deathclaws() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Each creature card in your graveyard has scavenge equal to its mana cost.",
            effect: StaticEffect::GraveyardCreaturesHaveScavenge,
        }],
        ..creature("Young Deathclaws", cost(&[generic(2), b(), g()]), vec![CreatureType::Lizard, CreatureType::Mutant], 4, 2)
    }
}

// ── Artifacts, enchantments, lands ──────────────────────────────────────────

/// Nuka-Nuke Launcher — {2} Artifact — Equipment. Equipped creature gets +3/+0
/// and has intimidate. Whenever equipped creature attacks, until the end of
/// defending player's next turn, that player gets two rad counters whenever
/// they cast a spell. Equip {3}.
pub fn nuka_nuke_launcher() -> CardDefinition {
    equipment(
        "Nuka-Nuke Launcher",
        cost(&[generic(2)]),
        cost(&[generic(3)]),
        EquipBonus {
            power: 3,
            keywords: vec![Keyword::Intimidate],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::RadOnCastUntilEndOfTheirNextTurn { who: PlayerRef::DefendingPlayer, amount: 2 },
            }],
            ..Default::default()
        },
    )
}

/// Power Fist — {1}{G} Artifact — Equipment. Equipped creature has trample and
/// "Whenever this creature deals combat damage to a player, put that many
/// +1/+1 counters on it." Equip {2}.
pub fn power_fist() -> CardDefinition {
    equipment(
        "Power Fist",
        cost(&[generic(1), g()]),
        cost(&[generic(2)]),
        EquipBonus {
            keywords: vec![Keyword::Trample],
            triggered_abilities: vec![on_combat_damage(plus(Selector::This, Value::TriggerEventAmount))],
            ..Default::default()
        },
    )
}

/// Recon Craft Theta — {4} Artifact — Vehicle 4/4. Flying. When it enters,
/// create a 0/0 blue Alien creature token and put a +1/+1 counter on it.
/// Whenever it attacks, proliferate. Crew 2.
pub fn recon_craft_theta() -> CardDefinition {
    CardDefinition {
        name: "Recon Craft Theta",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        triggered_abilities: vec![
            etb(make(
                TokenDefinition {
                    name: "Alien".into(),
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Blue],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Alien], ..Default::default() },
                    enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
                    ..Default::default()
                },
                Value::ONE,
            )),
            on_attack(Effect::Proliferate),
        ],
        ..Default::default()
    }
}

/// Strength Bobblehead — {3} Artifact — Bobblehead. {T}: Add one mana of any
/// color. {3}, {T}: Put X +1/+1 counters on target creature, where X is the
/// number of Bobbleheads you control. Activate only as a sorcery.
pub fn strength_bobblehead() -> CardDefinition {
    CardDefinition {
        name: "Strength Bobblehead",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Bobblehead], ..Default::default() },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                sorcery_speed: true,
                effect: plus(
                    target_filtered(R::Creature),
                    Value::CountOf(Box::new(Selector::EachPermanent(
                        R::HasArtifactSubtype(ArtifactSubtype::Bobblehead).and(R::ControlledByYou),
                    ))),
                ),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Struggle for Project Purity — {3}{U} Enchantment. As it enters, choose
/// Brotherhood or Enclave. Brotherhood — at the beginning of your upkeep, each
/// opponent draws a card; you draw a card for each card drawn this way.
/// Enclave — whenever a player attacks you with one or more creatures, that
/// player gets twice that many rad counters.
///
/// ⚠ Residual: Brotherhood draws you one card per opponent, not per card they
/// actually drew.
pub fn struggle_for_project_purity() -> CardDefinition {
    CardDefinition {
        name: "Struggle for Project Purity",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        enter_modes: Some(vec![
            EnterMode {
                label: "Brotherhood",
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                    effect: Effect::Seq(vec![
                        Effect::Draw { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                        draw(Value::CountOf(Box::new(Selector::Player(PlayerRef::EachOpponent)))),
                    ]),
                }],
                ..Default::default()
            },
            EnterMode {
                label: "Enclave",
                triggered_abilities: vec![TriggeredAbility {
                    // Fires once per attacking creature: two rad counters each.
                    event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
                    effect: Effect::AddRadCounters {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                        amount: Value::Const(2),
                    },
                }],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Vault 12: The Necropolis — {4}{B}{B} Enchantment — Saga. I — Each player
/// gets three rad counters. II — Create X 2/2 black Zombie Mutant tokens,
/// where X is the total number of rad counters among players. III — Put two
/// +1/+1 counters on each Zombie or Mutant you control.
pub fn vault_12_the_necropolis() -> CardDefinition {
    saga("Vault 12: The Necropolis", cost(&[generic(4), b(), b()]), vec![
        (1, rad(PlayerRef::EachPlayer, 3)),
        (2, make(zombie_mutant(), Value::RadCountersAmong(PlayerRef::EachPlayer))),
        (
            3,
            plus(
                Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(zombie_or_mutant())),
                Value::Const(2),
            ),
        ),
    ])
}

/// Vault 87: Forced Evolution — {3}{G}{U} Enchantment — Saga. I — Gain control
/// of target non-Mutant creature for as long as you control this Saga. II —
/// Put a +1/+1 counter on target creature you control; it becomes a Mutant in
/// addition to its other types. III — Draw cards equal to the greatest power
/// among Mutants you control.
pub fn vault_87_forced_evolution() -> CardDefinition {
    saga("Vault 87: Forced Evolution", cost(&[generic(3), g(), u()]), vec![
        (
            1,
            Effect::GainControlWhileYouControlSource {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Mutant))))),
            },
        ),
        (
            2,
            Effect::Seq(vec![
                plus(target_filtered(R::Creature.and(R::ControlledByYou)), Value::ONE),
                Effect::AddCreatureTypes {
                    what: Selector::Target(0),
                    creature_types: vec![CreatureType::Mutant],
                    duration: Duration::Permanent,
                },
            ]),
        ),
        (
            3,
            draw(Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(R::HasCreatureType(
                CreatureType::Mutant,
            ))))),
        ),
    ])
}

/// Mariposa Military Base — Land. You may have it enter tapped; if you do,
/// you get two rad counters. {T}: Add {C}. {5}, {T}: Draw a card; this costs
/// {1} less for each rad counter you have.
pub fn mariposa_military_base() -> CardDefinition {
    CardDefinition {
        name: "Mariposa Military Base",
        card_types: vec![CardType::Land],
        as_enters_effect: Some(Effect::AsEntersChooseMode(vec![
            Effect::Noop,
            Effect::Seq(vec![Effect::SourceEntersTapped, rad(PlayerRef::You, 2)]),
        ])),
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                cost_reduction_value: Some(Value::RadCountersAmong(PlayerRef::You)),
                effect: draw(Value::ONE),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Instants ────────────────────────────────────────────────────────────────

/// Atomize — {2}{B}{G} Instant. Destroy target nonland permanent. Proliferate.
pub fn atomize() -> CardDefinition {
    spell(
        "Atomize",
        cost(&[generic(2), b(), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Permanent.and(R::Nonland)) },
            Effect::Proliferate,
        ]),
    )
}

/// Mutational Advantage — {1}{G}{U} Instant. Permanents you control with
/// counters on them gain hexproof and indestructible until end of turn.
/// Prevent all damage that would be dealt to those permanents this turn.
/// Proliferate.
pub fn mutational_advantage() -> CardDefinition {
    let countered = || Selector::EachPermanent(R::ControlledByYou.and(R::WithAnyCounter));
    let give = |keyword| Effect::GrantKeyword { what: countered(), keyword, duration: Duration::EndOfTurn };
    spell(
        "Mutational Advantage",
        cost(&[generic(1), g(), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            give(Keyword::Hexproof),
            give(Keyword::Indestructible),
            give(Keyword::PreventDamageFromMatching(Box::new(R::Any))),
            Effect::Proliferate,
        ]),
    )
}

/// Radstorm — {3}{U} Instant. Storm. Proliferate.
pub fn radstorm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..spell("Radstorm", cost(&[generic(3), u()]), CardType::Instant, Effect::Proliferate)
    }
}
