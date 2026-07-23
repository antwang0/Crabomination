//! Dragon's Maze (DGM) gap cards, wave 3 — guild legends/mythics and remaining
//! multicolor spells on existing (or newly-added) primitives. Tests in
//! `classic_sets/dgm`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, SplitCard, SplitHalf, StaticAbility, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::mana::{b, cost, generic, r, u, w, Color};

/// A 1/1 red-and-white Soldier with haste (Blaze Commando's token).
fn boros_soldier() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        subtypes: creatures(vec![CreatureType::Soldier]),
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

/// Blaze Commando — {3}{R}{W} 5/3 Minotaur Soldier. Whenever an instant or
/// sorcery spell you control deals damage, create two 1/1 red-and-white Soldier
/// tokens with haste.
pub fn blaze_commando() -> CardDefinition {
    CardDefinition {
        name: "Blaze Commando",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Minotaur, CreatureType::Soldier]),
        power: 5,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::YourInstantOrSorceryDealtDamage,
                EventScope::YourControl,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: boros_soldier(),
            },
        }],
        ..Default::default()
    }
}

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}

/// Showstopper — {1}{B}{R} Instant. Until end of turn, creatures you control
/// gain "When this creature dies, it deals 2 damage to target creature an
/// opponent controls."
pub fn showstopper() -> CardDefinition {
    let death = TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            amount: Value::Const(2),
        },
    };
    CardDefinition {
        name: "Showstopper",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GrantTriggeredAbility {
            what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
            trigger: Box::new(death),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Teysa, Envoy of Ghosts — {5}{W}{B} 4/4 legendary. Vigilance, protection from
/// creatures. Whenever a creature deals combat damage to you, destroy it and
/// create a 1/1 white-black Spirit with flying.
pub fn teysa_envoy_of_ghosts() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Black],
        subtypes: creatures(vec![CreatureType::Spirit]),
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Teysa, Envoy of Ghosts",
        cost: cost(&[generic(5), w(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Advisor]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::ProtectionFromCreatures],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ControllerDealtCombatDamage, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Destroy { what: Selector::TriggerSource },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: spirit,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Scab-Clan Giant — {4}{R}{G} 4/5 Giant Warrior. When it enters, it fights
/// target creature an opponent controls. (The printed "chosen at random" is
/// approximated by a normal target.)
pub fn scab_clan_giant() -> CardDefinition {
    CardDefinition {
        name: "Scab-Clan Giant",
        cost: cost(&[generic(4), r(), crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Giant, CreatureType::Warrior]),
        power: 4,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::Fight {
            attacker: Selector::This,
            defender: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        })],
        ..Default::default()
    }
}

/// Breaking // Entering — {U}{B} // {4}{B}{R} Sorcery // Sorcery, Fuse.
/// Breaking: target player mills eight cards. Entering: put a creature card from
/// a graveyard onto the battlefield under your control; it gains haste.
pub fn breaking_entering() -> CardDefinition {
    CardDefinition {
        name: "Breaking // Entering",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Mill {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(8),
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(4), b(), r()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(R::Creature.and(R::InGraveyard)),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
            fuse: true,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Council of the Absolute — {2}{W}{U} 2/4 Human Advisor. As it enters, choose a
/// noncreature, nonland card name. Your opponents can't cast spells with the
/// chosen name; your spells with that name cost {2} less.
pub fn council_of_the_absolute() -> CardDefinition {
    CardDefinition {
        name: "Council of the Absolute",
        cost: cost(&[generic(2), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Advisor]),
        power: 2,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "Your opponents can't cast spells with the chosen name.",
                effect: StaticEffect::OpponentsCantCastNamed,
            },
            StaticAbility {
                description: "Spells with the chosen name you cast cost {2} less to cast.",
                effect: StaticEffect::NamedSpellCostReduction { amount: 2 },
            },
        ],
        triggered_abilities: vec![etb(Effect::NameCard { what: Selector::This })],
        ..Default::default()
    }
}
