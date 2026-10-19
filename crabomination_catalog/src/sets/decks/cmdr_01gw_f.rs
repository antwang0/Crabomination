//! Commander: 2026-10-19 (`01GwrDKP`) closers, sixth batch (5-7-gap tier) —
//! Gorion, Wise Mentor's Adventures. Tests in `tests/recent_b/cmdr_01gw_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{cost, g, generic, u, w, ManaCost};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CardType>, subtypes: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: types,
        subtypes: Subtypes { creature_types: subtypes, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Gorion, Wise Mentor — {G}{W}{U} 3/4 vigilant legendary Human Wizard.
/// Whenever you cast an Adventure spell, you may copy it (new targets allowed).
pub fn gorion_wise_mentor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::CastSpellIsAdventure),
            effect: Effect::MayDo {
                description: "Copy the Adventure spell?".into(),
                body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE }),
            },
        }],
        ..creature(
            "Gorion, Wise Mentor",
            cost(&[g(), w(), u()]),
            vec![CardType::Creature],
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Edgewall Inn — land, enters tapped; as it enters choose a color, {T} adds
/// it. {3}, {T}, sacrifice: return target card with an Adventure from your
/// graveyard to your hand.
pub fn edgewall_inn() -> CardDefinition {
    CardDefinition {
        name: "Edgewall Inn",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![etb(Effect::ChooseColorForSelf)],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: crate::effect::ManaPayload::ChosenColorOfSource },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::HasAdventure.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Edgewall Innkeeper — {G} 1/1 Human Peasant. Whenever you cast a creature
/// spell that has an Adventure, draw a card.
pub fn edgewall_innkeeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Creature.and(R::HasAdventure))),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature("Edgewall Innkeeper", cost(&[g()]), vec![CardType::Creature], vec![CreatureType::Human, CreatureType::Peasant], 1, 1)
    }
}

/// Moonshae Pixie // Pixie Dust — {3}{U} 2/2 flying Faerie; ETB, draw a card
/// per opponent dealt combat damage this turn. Adventure {1}{U} instant: up to
/// three target creatures gain flying until end of turn.
pub fn moonshae_pixie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::PlayersDealtCombatDamageThisTurn(PlayerRef::EachOpponent),
        })],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Pixie Dust",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::ApplyToTargets {
                max_targets: 3,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                }),
            },
        })),
        ..creature("Moonshae Pixie", cost(&[generic(3), u()]), vec![CardType::Creature], vec![CreatureType::Faerie], 2, 2)
    }
}

/// Sentinel of Lost Lore — {1}{G}{G} 3/4 Elf Knight. ETB, choose one or more:
/// return target exiled card you own with an Adventure to your hand; put
/// target exiled card you don't own with an Adventure on the bottom of its
/// owner's library; exile target player's graveyard.
pub fn sentinel_of_lost_lore() -> CardDefinition {
    let adventure_in_exile = || R::HasAdventure.and(R::InExile);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseN {
            picks: vec![0, 1, 2],
            modes: vec![
                Effect::Move {
                    what: target_filtered(adventure_in_exile().and(R::OwnedByYou)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Move {
                    what: target_filtered(adventure_in_exile().and(R::OwnedByYou.negate())),
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                        pos: crate::effect::LibraryPosition::Bottom,
                    },
                },
                Effect::TargetPlayerThen {
                    filter: R::Player,
                    then: Box::new(Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0), filter: None }),
                },
            ],
            up_to: None,
        })],
        ..creature("Sentinel of Lost Lore", cost(&[generic(1), g(), g()]), vec![CardType::Creature], vec![CreatureType::Elf, CreatureType::Knight], 3, 4)
    }
}
