//! A fourth wave of staples — reanimator pieces, sweepers, and lock pieces
//! that filled remaining gaps (Recurring Nightmare, Survival of the Fittest,
//! Living Death, Show and Tell, Smokestack, Tangle Wire, …).
//! Each card has a functionality test in `crabomination/src/tests/recent4.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement, Selector, SpellSubtype, StaticAbility,
    Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{DelayedTriggerKind, Duration, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, u, w, x, Color};

/// Ritual of Soot — {2}{B}{B} Sorcery. Destroy all creatures with mana value
/// 3 or less.
pub fn ritual_of_soot() -> CardDefinition {
    CardDefinition {
        name: "Ritual of Soot",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(3)),
            ),
        },
        ..Default::default()
    }
}

/// Recurring Nightmare — {2}{B} Enchantment. "Sacrifice a creature, Return
/// this enchantment to its owner's hand: Return target creature card from your
/// graveyard to the battlefield. Activate only as a sorcery."
pub fn recurring_nightmare() -> CardDefinition {
    CardDefinition {
        name: "Recurring Nightmare",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            return_self_cost: true,
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Survival of the Fittest — {1}{G} Enchantment. "{G}, Discard a creature
/// card: Search your library for a creature card, reveal it, put it into your
/// hand, then shuffle." (The reveal is cosmetic.)
pub fn survival_of_the_fittest() -> CardDefinition {
    CardDefinition {
        name: "Survival of the Fittest",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            discard_cost: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Footsteps of the Goryo — {2}{B} Sorcery — Arcane. Return target creature
/// card from your graveyard to the battlefield; sacrifice it at the beginning
/// of the next end step.
pub fn footsteps_of_the_goryo() -> CardDefinition {
    CardDefinition {
        name: "Footsteps of the Goryo",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Arcane], ..Default::default() },
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::SacrificePermanent { what: Selector::Target(0) }),
            },
        ]),
        ..Default::default()
    }
}

/// Apprentice Necromancer — {1}{B} 1/1 Zombie Wizard. "{B}, {T}, Sacrifice
/// this creature: Return target creature card from your graveyard to the
/// battlefield. That creature gains haste. At the beginning of the next end
/// step, sacrifice it."
pub fn apprentice_necromancer() -> CardDefinition {
    CardDefinition {
        name: "Apprentice Necromancer",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[b()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::SacrificePermanent { what: Selector::Target(0) }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Deafening Silence — {W} Enchantment. Each player can't cast more than one
/// noncreature spell each turn.
pub fn deafening_silence() -> CardDefinition {
    CardDefinition {
        name: "Deafening Silence",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Each player can't cast more than one noncreature spell each turn.",
            effect: StaticEffect::OneNoncreatureSpellPerTurn,
        }],
        ..Default::default()
    }
}

/// Ethersworn Canonist — {1}{W} 2/2 Human Cleric artifact creature. Each
/// player who has cast a nonartifact spell this turn can't cast additional
/// nonartifact spells.
pub fn ethersworn_canonist() -> CardDefinition {
    CardDefinition {
        name: "Ethersworn Canonist",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Each player who has cast a nonartifact spell this turn can't cast \
                additional nonartifact spells.",
            effect: StaticEffect::OneNonartifactSpellPerTurn,
        }],
        ..Default::default()
    }
}

/// Defense Grid — {2} Artifact. Each spell costs {3} more to cast except during
/// its controller's turn.
pub fn defense_grid() -> CardDefinition {
    CardDefinition {
        name: "Defense Grid",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Each spell costs {3} more to cast except during its controller's turn.",
            effect: StaticEffect::SpellsCostMoreExceptOnControllerTurn { amount: 3 },
        }],
        ..Default::default()
    }
}

/// Bontu's Last Reckoning — {1}{B}{B} Sorcery. Destroy all creatures. Lands you
/// control don't untap during your next untap step.
pub fn bontus_last_reckoning() -> CardDefinition {
    CardDefinition {
        name: "Bontu's Last Reckoning",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: Selector::EachPermanent(SelectionRequirement::Creature) },
            Effect::LandsDontUntapNextUntapStep { who: Selector::You },
        ]),
        ..Default::default()
    }
}

/// Syphon Mind — {3}{B} Sorcery. Each other player discards a card; you draw a
/// card for each card discarded this way.
pub fn syphon_mind() -> CardDefinition {
    CardDefinition {
        name: "Syphon Mind",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
                random: false,
            },
            Effect::Draw { who: Selector::You, amount: Value::CardsDiscardedThisEffect },
        ]),
        ..Default::default()
    }
}

/// Prosperity — {X}{U} Sorcery. Each player draws X cards.
pub fn prosperity() -> CardDefinition {
    CardDefinition {
        name: "Prosperity",
        cost: cost(&[x(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// Ondu Giant — {3}{G} 2/4 Giant Druid. ETB: you may search your library for a
/// basic land card and put it onto the battlefield tapped.
pub fn ondu_giant() -> CardDefinition {
    CardDefinition {
        name: "Ondu Giant",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        }],
        ..Default::default()
    }
}

/// Roiling Regrowth — {2}{G} Instant. Sacrifice a land. Search your library for
/// up to two basic land cards, put them onto the battlefield tapped.
pub fn roiling_regrowth() -> CardDefinition {
    CardDefinition {
        name: "Roiling Regrowth",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: SelectionRequirement::Land,
            },
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Roar of the Wurm — {6}{G} Sorcery. Create a 6/6 green Wurm token. Flashback
/// {3}{G}.
pub fn roar_of_the_wurm() -> CardDefinition {
    CardDefinition {
        name: "Roar of the Wurm",
        cost: cost(&[generic(6), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), g()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Wurm".into(),
                power: 6,
                toughness: 6,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Wurm],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Chart a Course — {1}{U} Sorcery. Draw two cards. Then discard a card unless
/// you attacked this turn.
pub fn chart_a_course() -> CardDefinition {
    CardDefinition {
        name: "Chart a Course",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::If {
                cond: Predicate::PlayerAttackedThisTurn { who: PlayerRef::You },
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                }),
            },
        ]),
        ..Default::default()
    }
}

