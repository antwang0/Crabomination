//! Edge of Eternities (EOE) gap closure. Tests in `classic_sets/eoe2`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TriggeredAbility, WardCost,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::{etb, on_attack, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, w};

/// "Void — At the beginning of your end step, sacrifice this unless a nonland
/// permanent left the battlefield or a spell was warped this turn."
fn void_upkeep_tax() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
            .with_filter(Predicate::Not(Box::new(Predicate::VoidActive {
                who: PlayerRef::You,
            }))),
        effect: Effect::SacrificeSource,
    }
}

/// Chorale of the Void — {3}{B} Aura. Enchant creature you control; its attacks
/// reanimate a creature from the defending player's graveyard tapped and
/// attacking. Void — sacrifice it at your end step if Void isn't on.
pub fn chorale_of_the_void() -> CardDefinition {
    CardDefinition {
        name: "Chorale of the Void",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![on_attack(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        R::Creature.and(R::InGraveyard).and(R::OwnedByDefendingPlayer),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::JoinCombatAttacking { what: Selector::LastMoved },
            ]))],
            ..Default::default()
        }),
        triggered_abilities: vec![void_upkeep_tax()],
        ..Default::default()
    }
}

/// Famished Worldsire — {5}{G}{G}{G} 0/0 Leviathan with ward {3} and devour
/// land 3; on entry it digs its own power deep for lands.
pub fn famished_worldsire() -> CardDefinition {
    CardDefinition {
        name: "Famished Worldsire",
        cost: cost(&[generic(5), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Leviathan],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(3)])))],
        as_enters_effect: Some(crate::effect::shortcut::devour_filter(3, R::Land)),
        triggered_abilities: vec![
            etb(Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::PowerOf(Box::new(Selector::This)),
                pick_filter: Some(R::Land),
                take: Some(Value::PowerOf(Box::new(Selector::This))),
                to_battlefield: true,
                optional: true,
                rest_to_graveyard: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
                rest_to_exile: false,
                then_if_picked: None,
            }),
        ],
        ..Default::default()
    }
}

/// Lightstall Inquisitor — {W} 2/1 Angel Wizard with vigilance. On entry each
/// opponent exiles a card from hand and may play it, taxed {1}.
pub fn lightstall_inquisitor() -> CardDefinition {
    CardDefinition {
        name: "Lightstall Inquisitor",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::EachOpponentExilesHandCardMayPlay { surcharge: 1 })],
        ..Default::default()
    }
}

/// Requiem Monolith — {2}{B} Artifact. {T}: a creature gains "whenever this is
/// dealt damage, draw that many cards and lose that much life" until end of
/// turn, and its controller may have the Monolith ping it.
pub fn requiem_monolith() -> CardDefinition {
    let bleed = TriggeredAbility {
        event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
            Effect::LoseLife { who: Selector::You, amount: Value::TriggerEventAmount },
        ]),
    };
    CardDefinition {
        name: "Requiem Monolith",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::GrantTriggeredAbility {
                    what: target_filtered(R::Creature),
                    trigger: Box::new(bleed),
                    duration: Duration::EndOfTurn,
                },
                Effect::MayDoBy {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    description: "Have Requiem Monolith deal 1 damage to that creature?".into(),
                    body: Box::new(Effect::DealDamage {
                        to: Selector::Target(0),
                        amount: Value::ONE,
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sothera, the Supervoid — {2}{B}{B} Legendary Enchantment. Your creatures'
/// deaths eat one of each opponent's; once a player is creatureless it cashes
/// itself in for an exiled creature with two extra +1/+1 counters.
pub fn sothera_the_supervoid() -> CardDefinition {
    CardDefinition {
        name: "Sothera, the Supervoid",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
                effect: Effect::EachOpponentExilesOwnCreature,
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::AnyPlayerControlsNoCreatures),
                effect: Effect::Seq(vec![
                    Effect::SacrificeSource,
                    Effect::Move {
                        what: Selector::Take {
                            inner: Box::new(Selector::CardExiledWithSource),
                            count: Box::new(Value::ONE),
                        },
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                    Effect::AddCounter {
                        what: Selector::LastMoved,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// The Dominion Bracelet — {2} Legendary Equipment. +1/+1 and a {15} exile
/// ability (cheaper by the bearer's power) that hands you the opponent's turn.
pub fn the_dominion_bracelet() -> CardDefinition {
    CardDefinition {
        name: "The Dominion Bracelet",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(15)]),
                sorcery_speed: true,
                exile_attachment_cost: true,
                cost_reduction_per_equipped_power: true,
                effect: Effect::ControlPlayerNextTurn { who: PlayerRef::Target(0) },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Moonlit Meditation — {2}{U} Aura. Enchant artifact or creature you control.
/// The first batch of tokens you'd create each turn may instead be copies of
/// the enchanted permanent.
pub fn moonlit_meditation() -> CardDefinition {
    CardDefinition {
        name: "Moonlit Meditation",
        cost: cost(&[generic(2), crate::mana::u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Artifact.or(R::Creature).and(R::ControlledByYou)),
        },
        static_abilities: vec![StaticAbility {
            description: "The first time you would create one or more tokens each turn, \
                          you may instead create that many tokens that are copies of \
                          enchanted permanent.",
            effect: StaticEffect::FirstTokensEachTurnBecomeCopiesOfAttached,
        }],
        ..Default::default()
    }
}
