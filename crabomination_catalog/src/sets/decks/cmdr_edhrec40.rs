//! Commander: the 2026-10-13 rescan's gaps for Braids, Conjurer Adept;
//! Taigam; Carth the Lion (and, in later batches, Gogo, Edea, Uugguu and
//! Loot). Tests in `tests/recent_b/cmdr_edhrec40.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{ActivationTrait, Effect, PlayerRef, VoteOption};
use crate::mana::{b, cost, g, generic, u, Color};
use crabomination_base::tokens::treasure_token;
use crabomination_base::turn_step::TurnStep;
use std::sync::Arc;

// ── Braids, Conjurer Adept / Taigam / Gogo ───────────────────────────────────

/// Shadow of the Second Sun — {4}{U}{U} Aura, enchant player. At the beginning
/// of each of the enchanted player's postcombat main phases, there is an
/// additional beginning phase after this phase (CR 500.8).
pub fn shadow_of_the_second_sun() -> CardDefinition {
    CardDefinition {
        name: "Shadow of the Second Sun",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::AnyPlayer)
                .with_filter(Predicate::IsTurnOf(PlayerRef::EnchantedPlayer)),
            effect: Effect::AdditionalBeginningPhase { count: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Jhoira's Timebug — {2} 1/2 artifact Insect. {T}: a permanent you control or
/// a suspended card you own with a time counter loses one or gains another, or
/// neither (the target filter asks for the time counter the effect reads).
pub fn jhoiras_timebug() -> CardDefinition {
    let target = || {
        target_filtered(
            R::OnBattlefield
                .and(R::ControlledByYou)
                .and(R::WithCounter(CounterType::Time))
                .or(R::InExile.and(R::IsSuspended).and(R::OwnedByYou)),
        )
    };
    CardDefinition {
        name: "Jhoira's Timebug",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PlayerChoosesOne {
                who: PlayerRef::You,
                options: vec![
                    VoteOption::new(
                        "Remove a time counter",
                        Effect::RemoveCounter { what: target(), kind: CounterType::Time, amount: Value::ONE },
                    ),
                    VoteOption::new(
                        "Put another time counter",
                        Effect::AddCounter { what: target(), kind: CounterType::Time, amount: Value::ONE },
                    ),
                    VoteOption::new("Leave it", Effect::Noop),
                ],
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Carth the Lion ───────────────────────────────────────────────────────────

fn legendary_enchantment(name: &'static str, mana: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}

/// Way of the Necromancer — {1}{B} legendary enchantment. Enters: empower
/// Jace 2. Whenever a creature you control dies, each planeswalker you control
/// gets a loyalty counter.
pub fn way_of_the_necromancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::EmpowerJace { who: PlayerRef::You, count: Value::Const(2) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(R::Planeswalker.and(R::ControlledByYou)),
                    kind: CounterType::Loyalty,
                    amount: Value::ONE,
                },
            },
        ],
        ..legendary_enchantment("Way of the Necromancer", cost(&[generic(1), b()]))
    }
}

/// Way of the Paradox — {2}{G} legendary enchantment. Enters: empower Jace 5.
/// Whenever you activate a loyalty ability (CR 606.1), you gain 1 life and may
/// play an additional land this turn.
pub fn way_of_the_paradox() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::EmpowerJace { who: PlayerRef::You, count: Value::Const(5) }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::AbilityActivatedWith(ActivationTrait::Loyalty),
                    EventScope::YourControl,
                ),
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                    Effect::GrantExtraLandPlay { who: PlayerRef::You, count: Value::ONE },
                ]),
            },
        ],
        ..legendary_enchantment("Way of the Paradox", cost(&[generic(2), g()]))
    }
}

/// Vraska, Relic Seeker — {4}{B}{G} planeswalker, loyalty 6. +2: a 2/2 black
/// Pirate with menace; −3: destroy an artifact, creature or enchantment and
/// make a Treasure; −10: target player's life total becomes 1.
pub fn vraska_relic_seeker() -> CardDefinition {
    let pirate = TokenDefinition {
        name: "Pirate".into(),
        power: 2,
        toughness: 2,
        colors: vec![Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pirate], ..Default::default() },
        keywords: vec![Keyword::Menace],
        ..Default::default()
    };
    CardDefinition {
        name: "Vraska, Relic Seeker",
        cost: cost(&[generic(4), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Vraska], ..Default::default() },
        base_loyalty: 6,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(pirate) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::Destroy {
                        what: target_filtered(R::Artifact.or(R::Creature).or(R::HasCardType(CardType::Enchantment))),
                    },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(treasure_token()),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -10,
                effect: Effect::SetLifeTotal { who: target_filtered(R::Player), amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
