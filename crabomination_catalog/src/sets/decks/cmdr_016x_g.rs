//! Commander: 2026-10-19 (`016Xh16H`) — Tesak, Judith's Hellhound's Dogs and
//! Zhang He, Wei General's soldiers. Tests in `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{blocks, etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn firebreathing(mana: ManaCost, power: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

fn dog() -> Vec<CreatureType> {
    vec![CreatureType::Elemental, CreatureType::Dog]
}

fn soldier() -> Vec<CreatureType> {
    vec![CreatureType::Human, CreatureType::Soldier]
}

// ── Tesak, Judith's Hellhound ───────────────────────────────────────────────

/// Ashmouth Hound — {1}{R} 2/1. Blocking or blocked, it deals 1 damage to
/// each creature it's in combat with.
pub fn ashmouth_hound() -> CardDefinition {
    let ping = |what: Selector| Effect::DealDamage { to: what, amount: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![
            blocks(ping(Selector::BlockedAttacker)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: ping(Selector::CreaturesInCombatWith(Box::new(Selector::This))),
            },
        ],
        ..creature("Ashmouth Hound", cost(&[generic(1), r()]), dog(), 2, 1)
    }
}

/// Bolt Hound — {2}{R} 2/2 haste; attacking, your other creatures get +1/+0.
pub fn bolt_hound() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            power: Value::ONE,
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Bolt Hound", cost(&[generic(2), r()]), dog(), 2, 2)
    }
}

/// Chandra's Magmutt — {1}{R} 2/2. {T}: 1 damage to target player or
/// planeswalker.
pub fn chandras_magmutt() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: target_filtered(R::Player.or(R::Planeswalker)), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Chandra's Magmutt", cost(&[generic(1), r()]), dog(), 2, 2)
    }
}

/// Deer-Dog — {1}{R} 1/3 first strike Elk Dog.
pub fn deer_dog() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        ..creature("Deer-Dog", cost(&[generic(1), r()]), vec![CreatureType::Elk, CreatureType::Dog], 1, 3)
    }
}

/// Fiery Hellhound — {1}{R}{R} 2/2; {R}: +1/+0.
pub fn fiery_hellhound() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![firebreathing(cost(&[r()]), 1)],
        ..creature("Fiery Hellhound", cost(&[generic(1), r(), r()]), dog(), 2, 2)
    }
}

/// Igneous Cur — {1}{R} 1/2; {1}{R}: +2/+0.
pub fn igneous_cur() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![firebreathing(cost(&[generic(1), r()]), 2)],
        ..creature("Igneous Cur", cost(&[generic(1), r()]), dog(), 1, 2)
    }
}

// ── Zhang He, Wei General ───────────────────────────────────────────────────

/// Nightsquad Commando — {2}{B} 2/3. ETB, if you attacked this turn, a 1/1
/// white Human Soldier (CR 603.4).
pub fn nightsquad_commando() -> CardDefinition {
    let token = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: soldier(), ..Default::default() },
        ..Default::default()
    };
    let mut t = etb(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(token) });
    t.event = t.event.with_filter(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You });
    CardDefinition {
        triggered_abilities: vec![t],
        ..creature("Nightsquad Commando", cost(&[generic(2), b()]), soldier(), 2, 3)
    }
}

/// Stone Catapult — {4}{B} 1/2. {T}: destroy target tapped nonblack creature;
/// only during your turn, before attackers are declared.
pub fn stone_catapult() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::Any(vec![
                    Predicate::CurrentStepIs(TurnStep::Upkeep),
                    Predicate::CurrentStepIs(TurnStep::Draw),
                    Predicate::CurrentStepIs(TurnStep::PreCombatMain),
                    Predicate::CurrentStepIs(TurnStep::BeginCombat),
                ]),
            ])),
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::Tapped).and(R::HasColor(Color::Black).negate())),
            },
            ..Default::default()
        }],
        ..creature("Stone Catapult", cost(&[generic(4), b()]), soldier(), 1, 2)
    }
}

/// Wei Ambush Force — {1}{B} 1/1; attacking, +2/+0.
pub fn wei_ambush_force() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Wei Ambush Force", cost(&[generic(1), b()]), soldier(), 1, 1)
    }
}

/// Wei Assassins — {3}{B}{B} 3/2. ETB target opponent chooses a creature
/// they control; destroy it.
pub fn wei_assassins() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::OpponentChoosesPermanentThen {
                filter: R::Creature,
                body: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                chooser: Some(PlayerRef::Target(0)),
                theirs: true,
                untargeted: true,
            }),
        })],
        ..creature(
            "Wei Assassins",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Assassin],
            3,
            2,
        )
    }
}

/// Wei Infantry — {1}{B} 2/1 Human Soldier.
pub fn wei_infantry() -> CardDefinition {
    creature("Wei Infantry", cost(&[generic(1), b()]), soldier(), 2, 1)
}

/// Young Wei Recruits — {1}{B} 2/2; can't block.
pub fn young_wei_recruits() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        ..creature("Young Wei Recruits", cost(&[generic(1), b()]), soldier(), 2, 2)
    }
}

/// Desperate Charge — {2}{B} sorcery. Creatures you control get +2/+0.
pub fn desperate_charge() -> CardDefinition {
    CardDefinition {
        name: "Desperate Charge",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Return to Battle — {B} sorcery. Return target creature card from your
/// graveyard to your hand.
pub fn return_to_battle() -> CardDefinition {
    CardDefinition {
        name: "Return to Battle",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(R::Creature.from_your_graveyard()),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}
