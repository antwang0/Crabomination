//! Commander: the 2026-10-16 rescan's two-card gaps, second leg (Wolverine,
//! Arno Dorian, Legolas, Leonardo, Donatello, Hulk, Moon-Boy, Doorman,
//! Hawkeye). Tests in `tests/recent_b/cmdr_oct16_a.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::card::{StaticAbility, StaticEffect};
use crate::effect::shortcut::{etb, heroic, sneak, target_filtered};
use crate::effect::{Duration, Effect, OpeningHandEffect, PlayerRef, Selector};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
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

fn legendary(d: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..d }
}

fn spell(name: &'static str, mana: ManaCost, instant: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if instant { CardType::Instant } else { CardType::Sorcery }],
        effect,
        ..Default::default()
    }
}

fn pump(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

fn turtle(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    legendary(creature(name, mana, vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle], p, t))
}

fn destroy_damaged() -> Effect {
    Effect::Destroy { what: target_filtered(R::Creature.and(R::DealtDamageThisTurn)) }
}

// ── Wolverine, Claws Out ────────────────────────────────────────────────────

/// Feral Ferocity — {1}{G} instant. Target creature gets +4/+4.
pub fn feral_ferocity() -> CardDefinition {
    spell("Feral Ferocity", cost(&[generic(1), g()]), true, pump(target_filtered(R::Creature), 4, 4))
}

// ── Arno Dorian ─────────────────────────────────────────────────────────────

/// Final-Sting Faerie — {3}{B} 2/2 flying Faerie Assassin. Enters: destroy
/// target creature that was dealt damage this turn.
pub fn final_sting_faerie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(destroy_damaged())],
        ..creature("Final-Sting Faerie", cost(&[generic(3), b()]), vec![CreatureType::Faerie, CreatureType::Assassin], 2, 2)
    }
}

/// Hooded Assassin — {2}{B} 1/2 Human Assassin. Enters: a +1/+1 counter on
/// it, or destroy target creature that was dealt damage this turn.
pub fn hooded_assassin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            destroy_damaged(),
        ]))],
        ..creature("Hooded Assassin", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Assassin], 1, 2)
    }
}

// ── Moon-Boy / Doorman ──────────────────────────────────────────────────────

/// Ravenous Sailback — {4}{G} 4/4 Dinosaur. Enters: haste this turn, or
/// destroy target artifact or enchantment.
pub fn ravenous_sailback() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
        ]))],
        ..creature("Ravenous Sailback", cost(&[generic(4), g()]), vec![CreatureType::Dinosaur], 3, 4)
    }
}

/// Aim High — {1}{G} instant. Untap target creature; +2/+2 and reach.
pub fn aim_high() -> CardDefinition {
    spell(
        "Aim High",
        cost(&[generic(1), g()]),
        true,
        Effect::Seq(vec![
            Effect::Untap { what: target_filtered(R::Creature), up_to: None },
            pump(Selector::Target(0), 2, 2),
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Reach, duration: Duration::EndOfTurn },
        ]),
    )
}

// ── Hulk, Strongest There Is ────────────────────────────────────────────────

/// Abomination, Irradiated Brute — {2}{G}{G} legendary 4/4 trample Gamma
/// Villain. A Gamma or Villain you control dealing combat damage to a player
/// puts that many +1/+1 counters on it.
pub fn abomination_irradiated_brute() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Gamma).or(R::HasCreatureType(CreatureType::Villain)),
                },
            ),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::TriggerEventAmount },
        }],
        ..creature("Abomination, Irradiated Brute", cost(&[generic(2), g(), g()]), vec![CreatureType::Gamma, CreatureType::Villain], 4, 4)
    })
}

/// Colossus, Steel Stalwart — {4}{G} legendary 4/5 Mutant Hero. Indestructible
/// during your turn; other Mutants you control get +1/+1.
pub fn colossus_steel_stalwart() -> CardDefinition {
    legendary(CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "During your turn, Colossus has indestructible.",
                effect: StaticEffect::WhileYourTurn {
                    inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Indestructible }),
                },
            },
            StaticAbility {
                description: "Other Mutants you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Mutant).and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    power: 1,
                    toughness: 1,
                },
            },
        ],
        ..creature("Colossus, Steel Stalwart", cost(&[generic(4), g()]), vec![CreatureType::Mutant, CreatureType::Hero], 4, 5)
    })
}

/// Prehistoric Turtlesaurus — {4}{G} 4/5 vigilance; costs {1} less if you
/// control a creature with a +1/+1 counter.
pub fn prehistoric_turtlesaurus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast if you control a creature with a +1/+1 counter on it.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::WithCounter(CounterType::PlusOnePlusOne)),
                )),
                amount: 1,
            },
        }],
        ..creature(
            "Prehistoric Turtlesaurus",
            cost(&[generic(4), g()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle, CreatureType::Dinosaur],
            4,
            5,
        )
    }
}

// ── Legolas, Counter of Kills ───────────────────────────────────────────────

/// Gift of Strands — {3}{G} flash Aura, +3/+3. Enters: scry 2.
pub fn gift_of_strands() -> CardDefinition {
    CardDefinition {
        name: "Gift of Strands",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        equipped_bonus: Some(EquipBonus { power: 3, toughness: 3, ..Default::default() }),
        ..Default::default()
    }
}

/// Legolas, Master Archer — {1}{G}{G} legendary 1/4 reach Elf Archer. A spell
/// you cast targeting him: a +1/+1 counter. A spell you cast targeting a
/// creature you don't control: he deals damage equal to his power to up to
/// one target creature.
pub fn legolas_master_archer() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![
            heroic(Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellTargetsMatch(R::Creature.and(R::ControlledByOpponent))),
                effect: Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::DealDamage {
                        to: Selector::Target(0),
                        amount: Value::PowerOf(Box::new(Selector::This)),
                    }),
                },
            },
        ],
        ..creature("Legolas, Master Archer", cost(&[generic(1), g(), g()]), vec![CreatureType::Elf, CreatureType::Archer], 1, 4)
    })
}

// ── Leonardo, Big Brother ───────────────────────────────────────────────────

/// April O'Neil, Kunoichi Trainee — {1}{W} legendary 2/2 Human Ninja. Enters:
/// scry 2; can't be blocked by creatures with power 3 or greater.
pub fn april_oneil_kunoichi_trainee() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByPowerAtLeast(3)],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        ..creature("April O'Neil, Kunoichi Trainee", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Ninja], 2, 2)
    })
}

/// The Last Ronin's Technique — {3}{W} instant; sneak {1}{W}. Three 1/1 white
/// Ninja Turtle Spirits; sneaked, they enter tapped and attacking.
pub fn the_last_ronins_technique() -> CardDefinition {
    let spirit = || {
        Arc::new(TokenDefinition {
            name: "Ninja Turtle Spirit".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            colors: vec![Color::White],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Ninja, CreatureType::Turtle, CreatureType::Spirit],
                ..Default::default()
            },
            ..Default::default()
        })
    };
    CardDefinition {
        keywords: vec![Keyword::Sneak(cost(&[generic(1), w()]))],
        alternative_cost: Some(AlternativeCost {
            effect_override: Some(Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: spirit(),
                cleanup: Default::default(),
                defender: None,
            }),
            ..sneak(cost(&[generic(1), w()]))
        }),
        ..spell(
            "The Last Ronin's Technique",
            cost(&[generic(3), w()]),
            true,
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(3), definition: spirit() },
        )
    }
}

// ── April O'Neil // Donatello ───────────────────────────────────────────────

/// Donatello, Rad Scientist — {5}{U} legendary 5/6 vigilance. Enters: tap up
/// to three creatures your opponents control and stun each.
pub fn donatello_rad_scientist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::Creature.and(R::ControlledByOpponent),
            effect: Box::new(Effect::Seq(vec![
                Effect::Tap { what: Selector::Target(0) },
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
            ])),
        })],
        ..turtle("Donatello, Rad Scientist", cost(&[generic(5), u()]), 5, 6)
    }
}

/// Donatello, Turtle Techie — {3}{U} legendary 3/4. Enters: draw a card if you
/// control an artifact.
pub fn donatello_turtle_techie() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou))),
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            else_: Box::new(Effect::Noop),
        })],
        ..turtle("Donatello, Turtle Techie", cost(&[generic(3), u()]), 3, 4)
    }
}

// ── Hawkeye, Trick Shot ─────────────────────────────────────────────────────

/// Quicksilver, Brash Blur — {R} legendary 1/1 haste Mutant Hero; may begin
/// the game on the battlefield. Power-up {4}{R}: a +1/+1 counter and a double
/// strike counter.
pub fn quicksilver_brash_blur() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Haste],
        opening_hand: Some(OpeningHandEffect::StartInPlay { tapped: false, extra: Effect::Noop }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), r()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::AddKeywordCounter { what: Selector::This, keyword: Keyword::DoubleStrike, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..creature("Quicksilver, Brash Blur", cost(&[r()]), vec![CreatureType::Mutant, CreatureType::Hero], 1, 1)
    })
}

/// Repulsor Blast — {3}{R} sorcery; teamwork 2. 5 damage to target creature;
/// with teamwork, 2 more to its controller.
pub fn repulsor_blast() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower { filter: R::Creature.and(R::ControlledByYou), power: 2 }),
        ..spell(
            "Repulsor Blast",
            cost(&[generic(3), r()]),
            false,
            Effect::Seq(vec![
                Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(5) },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::DealDamage {
                        to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                        amount: Value::Const(2),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}
