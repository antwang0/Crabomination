//! Commander: the cards the **Wretched Ranks** precon (FDC, Ghoulcaller
//! Gisa) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_teval.rs` (the black-graveyard precon module).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::catalog::sets::tap_add;
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, generic};
use std::sync::Arc;

/// A 2/2 black Zombie, tapped or not.
fn zombie(tapped: bool) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        tapped,
        ..Default::default()
    })
}

fn zombies(count: Value, tapped: bool) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: zombie(tapped) }
}

fn sorcery(name: &'static str, mana: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

fn zombie_creature(name: &'static str, mana: crate::mana::ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Army of the Damned — thirteen tapped Zombies, and again by flashback.
pub fn army_of_the_damned() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(7), b(), b(), b()]))],
        ..sorcery(
            "Army of the Damned",
            cost(&[generic(5), b(), b(), b()]),
            zombies(Value::Const(13), true),
        )
    }
}

/// Endless Ranks of the Dead — each upkeep, half your Zombies again
/// (rounded down).
pub fn endless_ranks_of_the_dead() -> CardDefinition {
    let your_zombies = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::ControlledByYou)),
        filter: R::HasCreatureType(CreatureType::Zombie),
    };
    CardDefinition {
        name: "Endless Ranks of the Dead",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: zombies(Value::HalfDown(Box::new(your_zombies)), false),
        }],
        ..Default::default()
    }
}

/// Infernal Idol — a black mana rock that cashes in for two cards and 2 life.
pub fn infernal_idol() -> CardDefinition {
    CardDefinition {
        name: "Infernal Idol",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            tap_add(Color::Black),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b(), b()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                    Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Josu Vess, Lich Knight — kicked, it brings eight menacing Zombie Knights.
pub fn josu_vess_lich_knight() -> CardDefinition {
    let knight = Arc::new(TokenDefinition {
        name: "Zombie Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Knight],
            ..Default::default()
        },
        keywords: vec![Keyword::Menace],
        ..Default::default()
    });
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Knight],
            ..Default::default()
        },
        keywords: vec![Keyword::Menace, Keyword::Kicker(cost(&[generic(5), b()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(8),
                definition: knight,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..zombie_creature("Josu Vess, Lich Knight", cost(&[generic(2), b(), b()]), 4, 5)
    }
}

/// Liliana's Reaver — a connecting hit costs the player a card and makes a
/// tapped Zombie.
pub fn lilianas_reaver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            // `Target(0)` is the damaged player (Hypnotic Specter's shape).
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                    random: false,
                },
                zombies(Value::ONE, true),
            ]),
        }],
        ..zombie_creature("Liliana's Reaver", cost(&[generic(2), b(), b()]), 4, 3)
    }
}

/// Necrotic Hex — everyone sacrifices six creatures; you get six tapped
/// Zombies.
pub fn necrotic_hex() -> CardDefinition {
    sorcery(
        "Necrotic Hex",
        cost(&[generic(6), b()]),
        Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::Const(6),
                filter: R::Creature,
            },
            zombies(Value::Const(6), true),
        ]),
    )
}

/// Open the Graves — a Zombie for each nontoken creature of yours that dies.
pub fn open_the_graves() -> CardDefinition {
    CardDefinition {
        name: "Open the Graves",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
            ),
            effect: zombies(Value::ONE, false),
        }],
        ..Default::default()
    }
}

/// Razorlash Transmogrant — a 3/1 that can't block and climbs back from the
/// graveyard with a +1/+1 counter, for {B}{B} against a nonbasic-heavy
/// opponent (CR 102.2: one opponent with four).
pub fn razorlash_transmogrant() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::CantBlock],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b(), b()]),
            from_graveyard: true,
            cost_reduction_if: Some((
                Predicate::AnOpponentControlsAtLeast { filter: R::IsNonbasicLand, n: 4 },
                4,
            )),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..zombie_creature("Razorlash Transmogrant", cost(&[generic(2)]), 3, 1)
    }
}

/// Syphon Flesh — each other player sacrifices a creature; a Zombie for each
/// one sacrificed.
pub fn syphon_flesh() -> CardDefinition {
    sorcery(
        "Syphon Flesh",
        cost(&[generic(4), b()]),
        Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Creature,
            },
            zombies(Value::SacrificedCount, false),
        ]),
    )
}
