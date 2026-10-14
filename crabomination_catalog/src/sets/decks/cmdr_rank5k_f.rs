//! Commander: four-card gaps of EDHREC's commanders ranked 4501-5500, second
//! batch (the 2026-10-14 rescan). Tests in `tests/recent_b/cmdr_rank5k_f.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, u, w};
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

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

/// "Activate only during your turn, before attackers are declared."
fn before_attackers() -> Option<Predicate> {
    Some(Predicate::All(vec![
        Predicate::IsTurnOf(PlayerRef::You),
        Predicate::Any(vec![
            Predicate::CurrentStepIs(TurnStep::Upkeep),
            Predicate::CurrentStepIs(TurnStep::Draw),
            Predicate::CurrentStepIs(TurnStep::PreCombatMain),
            Predicate::CurrentStepIs(TurnStep::BeginCombat),
        ]),
    ]))
}

fn tap_before_attackers(effect: Effect) -> ActivatedAbility {
    ActivatedAbility { tap_cost: true, condition: before_attackers(), effect, ..Default::default() }
}

fn advisor() -> Vec<CreatureType> {
    vec![CreatureType::Human, CreatureType::Advisor]
}

fn plus_one(what: Selector, n: i32) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: Value::Const(n) }
}

// ── Lu Su, Wu Advisor ────────────────────────────────────────────────────────

/// Lu Su, Wu Advisor — {3}{U}{U} legendary 1/2 Human Advisor. {T}: draw a
/// card, only during your turn before attackers are declared.
pub fn lu_su_wu_advisor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![tap_before_attackers(draw(1))],
        ..legend("Lu Su, Wu Advisor", cost(&[generic(3), u(), u()]), advisor(), 1, 2)
    }
}

/// Council of Advisors — {2}{U} 1/1 Human Advisor. When it enters, draw a card.
pub fn council_of_advisors() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(draw(1))],
        ..creature("Council of Advisors", cost(&[generic(2), u()]), advisor(), 1, 1)
    }
}

/// Talas Researcher — {4}{U} 1/1 Human Pirate Wizard. {T}: draw a card, only
/// during your turn before attackers are declared.
pub fn talas_researcher() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![tap_before_attackers(draw(1))],
        ..creature(
            "Talas Researcher",
            cost(&[generic(4), u()]),
            vec![CreatureType::Human, CreatureType::Pirate, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Zhuge Jin, Wu Strategist — {1}{U}{U} legendary 1/1 Human Advisor. {T}:
/// target creature can't be blocked this turn, only during your turn before
/// attackers are declared.
pub fn zhuge_jin_wu_strategist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![tap_before_attackers(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::Unblockable,
            duration: Duration::EndOfTurn,
        })],
        ..legend("Zhuge Jin, Wu Strategist", cost(&[generic(1), u(), u()]), advisor(), 1, 1)
    }
}

// ── Xun Yu, Wei Advisor ──────────────────────────────────────────────────────

/// Xun Yu, Wei Advisor — {1}{B}{B} legendary 1/1 Human Advisor. {T}: target
/// creature you control gets +2/+0 until end of turn, only during your turn
/// before attackers are declared.
pub fn xun_yu_wei_advisor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![tap_before_attackers(Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..legend("Xun Yu, Wei Advisor", cost(&[generic(1), b(), b()]), advisor(), 1, 1)
    }
}

/// Contagious Nim — {2}{B} 2/2 Phyrexian Zombie, infect.
pub fn contagious_nim() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        ..creature("Contagious Nim", cost(&[generic(2), b()]), vec![CreatureType::Phyrexian, CreatureType::Zombie], 2, 2)
    }
}

/// Swamp Mosquito — {1}{B} 0/1 flying Insect. Whenever it attacks and isn't
/// blocked, the defending player gets a poison counter.
pub fn swamp_mosquito() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
            effect: Effect::AddPoison { who: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::ONE },
        }],
        ..creature("Swamp Mosquito", cost(&[generic(1), b()]), vec![CreatureType::Insect], 0, 1)
    }
}

/// Relic Putrescence — {2}{B} Aura, enchant artifact. Whenever the enchanted
/// artifact becomes tapped, its controller gets a poison counter.
pub fn relic_putrescence() -> CardDefinition {
    CardDefinition {
        name: "Relic Putrescence",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Artifact) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::EnchantedBySource),
            effect: Effect::AddPoison {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

// ── Giant-Man, Gargantuan Genius ─────────────────────────────────────────────

/// Giant-Man, Gargantuan Genius — {3}{G}{G} legendary 6/6 reach Human
/// Scientist Hero. At the beginning of your first main phase, add {G} for each
/// creature you control with power 4 or greater.
pub fn giant_man_gargantuan_genius() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Green,
                    Value::count(Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                    )),
                ),
            },
        }],
        ..legend(
            "Giant-Man, Gargantuan Genius",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Hero],
            6,
            6,
        )
    }
}

/// Gigantic Big Bear — {5}{G}{G} 10/7 Bear; can't be countered; hexproof,
/// haste.
pub fn gigantic_big_bear() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered, Keyword::Hexproof, Keyword::Haste],
        ..creature("Gigantic Big Bear", cost(&[generic(5), g(), g()]), vec![CreatureType::Bear], 10, 7)
    }
}

/// Hulk, Brutal Brawler — {2}{G}{G} legendary 4/4 Gamma Berserker Hero. He
/// attacks each combat if able. Whenever he attacks, a +1/+1 counter on each
/// other creature you control.
pub fn hulk_brutal_brawler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustAttack],
        triggered_abilities: vec![on_attack(plus_one(
            Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            1,
        ))],
        ..legend(
            "Hulk, Brutal Brawler",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Gamma, CreatureType::Berserker, CreatureType::Hero],
            4,
            4,
        )
    }
}

/// Lizard, Connors's Curse — {2}{G}{G} legendary 5/5 trample Lizard Villain.
/// When he enters, up to one other target creature loses all abilities and
/// becomes a green Lizard creature with base power and toughness 4/4.
pub fn lizard_connorss_curse() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::LoseAllAbilities {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    duration: Duration::Permanent,
                },
                Effect::BecomeColor {
                    what: Selector::Target(0),
                    colors: vec![Color::Green],
                    duration: Duration::Permanent,
                    additive: false,
                },
                Effect::BecomeCreatureType {
                    what: Selector::Target(0),
                    creature_types: vec![CreatureType::Lizard],
                    duration: Duration::Permanent,
                },
                Effect::SetBasePT {
                    what: Selector::Target(0),
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    duration: Duration::Permanent,
                },
            ])),
        })],
        ..legend(
            "Lizard, Connors's Curse",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Lizard, CreatureType::Villain],
            5,
            5,
        )
    }
}

// ── Mister Hyde, Monster Within ──────────────────────────────────────────────

/// Knight of Wundagore — {1}{G} 2/1 trample Cat Knight Villain. Once each
/// turn, whenever a +1/+1 counter is put on another creature you control, a
/// +1/+1 counter on it.
pub fn knight_of_wundagore() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        // "Whenever you put a +1/+1 counter on another creature": read as a
        // counter landing on another creature you control (INCOMPLETE_CARDS).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                })
                .once_per_turn(),
            effect: plus_one(Selector::This, 1),
        }],
        ..creature(
            "Knight of Wundagore",
            cost(&[generic(1), g()]),
            vec![CreatureType::Cat, CreatureType::Knight, CreatureType::Villain],
            2,
            1,
        )
    }
}

/// Mirkwood Pathmaker — {2}{G} */* Elf Ranger; power and toughness each equal
/// to the number of lands you control.
pub fn mirkwood_pathmaker() -> CardDefinition {
    let lands = || Value::CountMatching { sel: Box::new(Selector::EachPermanent(R::ControlledByYou)), filter: R::Land };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Mirkwood Pathmaker's power and toughness are each equal to the number of lands you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: lands(), toughness: lands() },
        }],
        ..creature("Mirkwood Pathmaker", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Ranger], 0, 0)
    }
}

/// Shang-Chi, Martial Mentor — {3}{G}{G} legendary 4/4 Human Warrior Hero.
/// +1/+1 counters on your creatures come doubled. Power-up — {5}{G}{G}: three
/// +1/+1 counters on him.
pub fn shang_chi_martial_mentor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on a creature you control, twice that many +1/+1 counters are put on that creature instead.",
            effect: StaticEffect::DoublePlusOneCounters,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g(), g()]),
            power_up: true,
            effect: plus_one(Selector::This, 3),
            ..Default::default()
        }],
        ..legend(
            "Shang-Chi, Martial Mentor",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero],
            4,
            4,
        )
    }
}

/// Study the Classics — {2}{G} sorcery. A +1/+1 counter on target creature,
/// then double its +1/+1 counters; gain life equal to its +1/+1 counters.
pub fn study_the_classics() -> CardDefinition {
    CardDefinition {
        name: "Study the Classics",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            plus_one(target_filtered(R::Creature), 1),
            Effect::DoubleCountersOnEach { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::CountersOn { what: Box::new(Selector::Target(0)), kind: CounterType::PlusOnePlusOne },
            },
        ]),
        ..Default::default()
    }
}

// ── Amazing Spider-Girl ──────────────────────────────────────────────────────

/// Agent Phil Coulson — {1}{W} legendary 2/2 vigilance Human Spy Hero. {T}: a
/// +1/+1 counter on each other Hero you control.
pub fn agent_phil_coulson() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: plus_one(
                Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Hero).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                1,
            ),
            ..Default::default()
        }],
        ..legend(
            "Agent Phil Coulson",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Spy, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Kor Line-Slinger — {1}{W} 0/1 Kor Scout. {T}: tap target creature with
/// power 3 or less.
pub fn kor_line_slinger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature.and(R::PowerAtMost(3))) },
            ..Default::default()
        }],
        ..creature("Kor Line-Slinger", cost(&[generic(1), w()]), vec![CreatureType::Kor, CreatureType::Scout], 0, 1)
    }
}

/// Friendly Neighborhood — {3}{W} Aura, enchant land. It enters with three
/// 1/1 green and white Human Citizen tokens. Enchanted land has "{1}, {T}:
/// target creature gets +1/+1 until end of turn for each creature you
/// control", as a sorcery.
pub fn friendly_neighborhood() -> CardDefinition {
    let citizen = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: "Human Citizen".into(),
            power: 1,
            toughness: 1,
            colors: vec![Color::Green, Color::White],
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Citizen], ..Default::default() },
            ..Default::default()
        }),
    };
    let creatures = || Value::count(Selector::EachPermanent(R::Creature.and(R::ControlledByYou)));
    CardDefinition {
        name: "Friendly Neighborhood",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        triggered_abilities: vec![etb(Effect::Seq(vec![citizen(), citizen(), citizen()]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: creatures(),
                    toughness: creatures(),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Sudden Strike — {1}{W} instant. Destroy target attacking or blocking
/// creature.
pub fn sudden_strike() -> CardDefinition {
    CardDefinition {
        name: "Sudden Strike",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))) },
        ..Default::default()
    }
}

// ── Vhal, Candlekeep Researcher ──────────────────────────────────────────────

/// Vhal, Candlekeep Researcher — {3}{U} legendary 2/3 vigilance Human Wizard;
/// choose a Background. {T}: add {C} equal to her toughness, not spendable on
/// spells cast from your hand.
pub fn vhal_candlekeep_researcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::ChooseABackground],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::ToughnessOf(Box::new(Selector::This)))),
                    crate::mana::SpendRestriction::NotSpellsFromHand,
                ),
            },
            ..Default::default()
        }],
        ..legend(
            "Vhal, Candlekeep Researcher",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}
