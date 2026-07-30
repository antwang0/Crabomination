//! "Enchant player" Auras (CR 303.4a) — the Curse cycle and Psychic
//! Possession. Tests in `core_rules/cr_recent36`.

use crate::card::{
    CardDefinition, CardType, EnchantmentSubtype, EventKind, EventScope, EventSpec,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, Predicate, Selector};
use crate::mana::{ManaCost, b, cost, generic, r, u, w};
use crabomination_base::turn_step::TurnStep;

/// An "enchant player" Aura: `Effect::Attach` anchored to a target player.
fn player_aura(
    name: &'static str,
    mana: ManaCost,
    curse: bool,
    statics: Vec<StaticAbility>,
    triggered: Vec<TriggeredAbility>,
) -> CardDefinition {
    let mut subs = vec![EnchantmentSubtype::Aura];
    if curse {
        subs.push(EnchantmentSubtype::Curse);
    }
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: subs,
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Player),
        },
        static_abilities: statics,
        triggered_abilities: triggered,
        ..Default::default()
    }
}

/// "At the beginning of enchanted player's upkeep, …"
fn enchanted_player_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(TurnStep::Upkeep),
            EventScope::AnyPlayer,
        )
        .with_filter(Predicate::IsTurnOf(PlayerRef::EnchantedPlayer)),
        effect,
    }
}

/// Psychic Possession — {2}{U}{U}. Skip your draw step; draw whenever the
/// enchanted player draws.
pub fn psychic_possession() -> CardDefinition {
    player_aura(
        "Psychic Possession",
        cost(&[generic(2), u(), u()]),
        false,
        vec![StaticAbility {
            description: "Skip your draw step.",
            effect: StaticEffect::SkipStep {
                step: TurnStep::Draw,
                all_players: false,
            },
        }],
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::EnchantedBySource),
            effect: Effect::MayDo {
                description: "Draw a card".into(),
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
            },
        }],
    )
}

/// Curse of the Pierced Heart — {R}. 1 damage to the enchanted player each of
/// their upkeeps.
pub fn curse_of_the_pierced_heart() -> CardDefinition {
    player_aura(
        "Curse of the Pierced Heart",
        cost(&[r()]),
        true,
        vec![],
        vec![enchanted_player_upkeep(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EnchantedPlayer),
            amount: Value::ONE,
        })],
    )
}

/// Curse of Death's Hold — {3}{B}{B}. Creatures the enchanted player controls
/// get -1/-1.
pub fn curse_of_deaths_hold() -> CardDefinition {
    player_aura(
        "Curse of Death's Hold",
        cost(&[generic(3), b(), b()]),
        true,
        vec![StaticAbility {
            description: "Creatures enchanted player controls get -1/-1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::ControlledBy {
                    who: PlayerRef::EnchantedPlayer,
                    filter: R::Creature,
                },
                power: -1,
                toughness: -1,
            },
        }],
        vec![],
    )
}

/// Curse of Exhaustion — {2}{W}{W}. The enchanted player is limited to one
/// spell each turn.
pub fn curse_of_exhaustion() -> CardDefinition {
    player_aura(
        "Curse of Exhaustion",
        cost(&[generic(2), w(), w()]),
        true,
        vec![StaticAbility {
            description: "Enchanted player can't cast more than one spell each turn.",
            effect: StaticEffect::EnchantedPlayerOneSpellPerTurn,
        }],
        vec![],
    )
}

/// Curse of Bloodletting — {3}{R}. Damage to the enchanted player is doubled.
pub fn curse_of_bloodletting() -> CardDefinition {
    player_aura(
        "Curse of Bloodletting",
        cost(&[generic(3), r()]),
        true,
        vec![StaticAbility {
            description: "If a source would deal damage to enchanted player, it deals double that damage instead.",
            effect: StaticEffect::DoubleDamageToEnchantedPlayer,
        }],
        vec![],
    )
}

/// Cruel Reality — {5}{B}{B}. Each of the enchanted player's upkeeps they
/// sacrifice a creature or planeswalker, or lose 5 life.
pub fn cruel_reality() -> CardDefinition {
    player_aura(
        "Cruel Reality",
        cost(&[generic(5), b(), b()]),
        true,
        vec![],
        vec![enchanted_player_upkeep(Effect::If {
            cond: Predicate::SelectorExists(Selector::ControlledBy {
                who: PlayerRef::EnchantedPlayer,
                filter: R::Creature.or(R::Planeswalker),
            }),
            then: Box::new(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EnchantedPlayer),
                count: Value::ONE,
                filter: R::Creature.or(R::Planeswalker),
            }),
            else_: Box::new(Effect::LoseLife {
                who: Selector::Player(PlayerRef::EnchantedPlayer),
                amount: Value::Const(5),
            }),
        })],
    )
}
