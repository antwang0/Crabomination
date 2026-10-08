//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat — one or two per deck, gathered here
//! rather than beside a commander of their own. Tests in
//! `tests/recent_b/cmdr_edhrec.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color};
use std::sync::Arc;

fn creature_types(types: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: types, ..Default::default() }
}

fn token_1_1(name: &str, color: Color, kind: CreatureType) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: creature_types(vec![kind]),
        ..Default::default()
    }
}

fn mint(token: TokenDefinition, count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(token) }
}

/// Cadira, Caller of the Small — trample; combat damage to a player makes a
/// 1/1 Rabbit for each token you control.
pub fn cadira_caller_of_the_small() -> CardDefinition {
    let tokens = Value::count(Selector::EachPermanent(R::ControlledByYou.and(R::IsToken)));
    CardDefinition {
        name: "Cadira, Caller of the Small",
        cost: cost(&[generic(1), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Orc, CreatureType::Ranger]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: mint(token_1_1("Rabbit", Color::White, CreatureType::Rabbit), tokens),
        }],
        ..Default::default()
    }
}

/// The Unbeatable Squirrel Girl — a Squirrel on entering and attacking;
/// {1}{G}{G}{G} makes one for each Squirrel you control.
pub fn the_unbeatable_squirrel_girl() -> CardDefinition {
    let squirrel = || token_1_1("Squirrel", Color::Green, CreatureType::Squirrel);
    let squirrels = Value::count(Selector::EachPermanent(
        R::ControlledByYou.and(R::HasCreatureType(CreatureType::Squirrel)),
    ));
    CardDefinition {
        name: "The Unbeatable Squirrel Girl",
        cost: cost(&[generic(1), g(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Squirrel, CreatureType::Human, CreatureType::Hero]),
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(mint(squirrel(), Value::ONE)), on_attack(mint(squirrel(), Value::ONE))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g(), g(), g()]),
            effect: mint(squirrel(), squirrels),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Forced Fruition — each spell an opponent casts draws them seven.
pub fn forced_fruition() -> CardDefinition {
    CardDefinition {
        name: "Forced Fruition",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::Draw { who: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(7) },
        }],
        ..Default::default()
    }
}

/// Lesser Masticore — discard a card to cast it; {4}: 1 damage to target
/// creature; persist.
pub fn lesser_masticore() -> CardDefinition {
    CardDefinition {
        name: "Lesser Masticore",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Masticore]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Persist],
        additional_cast_cost: vec![AdditionalCastCost::Discard { count: 1, filter: None }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dark Deal — each player discards their hand and draws one fewer.
pub fn dark_deal() -> CardDefinition {
    CardDefinition {
        name: "Dark Deal",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DiscardHandDrawThatManyLess { who: Selector::Player(PlayerRef::EachPlayer), less: 1 },
        ..Default::default()
    }
}

/// Dream Stalker — entering returns a permanent you control to its owner's
/// hand.
pub fn dream_stalker() -> CardDefinition {
    CardDefinition {
        name: "Dream Stalker",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Illusion]),
        power: 1,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::ReturnOneYouControl { filter: R::Permanent, keep_best: false })],
        ..Default::default()
    }
}

/// Agent of Treachery — entering steals target permanent; your end step
/// draws three while you control three or more permanents you don't own.
pub fn agent_of_treachery() -> CardDefinition {
    let three_stolen = || Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::ControlledByYou.and(R::Not(Box::new(R::OwnedByYou)))),
        n: Value::Const(3),
    };
    CardDefinition {
        name: "Agent of Treachery",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Rogue]),
        power: 2,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::GainControl { what: target_filtered(R::Permanent), to: None, duration: Duration::Permanent }),
            TriggeredAbility {
                // CR 603.4 — checked as it triggers and again as it resolves.
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(three_stolen()),
                effect: Effect::If {
                    cond: three_stolen(),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(3) }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Worldfire — exile every permanent, hand and graveyard; each life total
/// becomes 1.
pub fn worldfire() -> CardDefinition {
    let zone = |zone| Selector::CardsInZone { who: PlayerRef::EachPlayer, zone, filter: R::Any };
    CardDefinition {
        name: "Worldfire",
        cost: cost(&[generic(6), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Exile { what: Selector::EachPermanent(R::Permanent) },
            Effect::Move { what: zone(crate::card::Zone::Hand), to: ZoneDest::Exile },
            Effect::Move { what: zone(crate::card::Zone::Graveyard), to: ZoneDest::Exile },
            Effect::SetLifeTotal { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Ganax, Astral Hunter — flying; it or another Dragon of yours entering
/// makes a Treasure. Choose a Background.
pub fn ganax_astral_hunter() -> CardDefinition {
    CardDefinition {
        name: "Ganax, Astral Hunter",
        cost: cost(&[generic(4), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Dragon]),
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::ChooseABackground],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Dragon),
                },
            ),
            effect: mint(crabomination_base::tokens::treasure_token(), Value::ONE),
        }],
        ..Default::default()
    }
}

/// Afterlife Insurance — your creatures gain afterlife 1 until end of turn;
/// draw a card.
pub fn afterlife_insurance() -> CardDefinition {
    CardDefinition {
        name: "Afterlife Insurance",
        cost: cost(&[generic(1), hybrid(Color::White, Color::Black)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantTriggeredAbility {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                trigger: Box::new(crate::effect::shortcut::afterlife(1)),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Dockside Chef — {1}{B}, sacrifice an artifact or creature: draw a card.
pub fn dockside_chef() -> CardDefinition {
    CardDefinition {
        name: "Dockside Chef",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Citizen]),
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((R::Artifact.or(R::Creature), 1)),
            sac_other_may_be_source: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}
