//! Additional Modern-staple instants not already covered by `instants.rs`:
//! enchantment removal, narrower counterspells, and Dovin's Veto's
//! "can't-be-countered" rider.

use crate::card::{
    CardDefinition, CardType, Effect, EventKind, EventScope, EventSpec,
    SelectionRequirement, StaticAbility, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{PlayerRef, Selector, StaticEffect, Value};
use crate::mana::{cost, g, generic, u, w};

/// Disenchant — {1}{W} Instant. Destroy target artifact or enchantment.
pub fn disenchant() -> CardDefinition {
    CardDefinition {
        name: "Disenchant",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
            ),
        },
        ..Default::default()
    }
}

/// Naturalize — {1}{G} Instant. Mirror of Disenchant in green.
pub fn naturalize() -> CardDefinition {
    CardDefinition {
        name: "Naturalize",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
            ),
        },
        ..Default::default()
    }
}

/// Nature's Claim — {G} Instant. Destroy target artifact or enchantment;
/// its controller gains 4 life.
pub fn natures_claim() -> CardDefinition {
    CardDefinition {
        name: "Nature's Claim",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(4),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Negate — {1}{U} Instant. Counter target noncreature spell.
pub fn negate() -> CardDefinition {
    CardDefinition {
        name: "Negate",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::Noncreature),
        },
        ..Default::default()
    }
}

/// Dispel — {U} Instant. Counter target instant spell.
pub fn dispel() -> CardDefinition {
    CardDefinition {
        name: "Dispel",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::HasCardType(CardType::Instant)),
        },
        ..Default::default()
    }
}

/// Dovin's Veto — {W}{U} Instant. Counter target noncreature spell. This
/// spell can't be countered. The "can't be countered" rider is encoded as
/// `Keyword::CantBeCountered`; `caster_grants_uncounterable` flags the spell
/// so `CounterSpell` and `CounterUnlessPaid` skip it on the stack.
pub fn dovins_veto() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Dovin's Veto",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::Noncreature),
        },
        ..Default::default()
    }
}

/// Static Prison — `{X}{2}{W}` Enchantment. Static Prison enters with X
/// stun counters on it. Tap target permanent. At the beginning of each
/// Static Prison — {W} Enchantment. ETB: exile target nonland permanent an
/// opponent controls until this leaves the battlefield, and you get {E}{E}. At
/// the beginning of your first main phase, sacrifice it unless you pay {E}.
pub fn static_prison() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Static Prison",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::ExileUntilSourceLeaves {
                    what: target_filtered(
                        SelectionRequirement::Nonland.and(SelectionRequirement::ControlledByOpponent),
                    ),
                    return_to: crate::card::ExileReturnZone::Battlefield,
                },
                Effect::AddEnergy(Value::Const(2)),
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
                effect: Effect::PayEnergyOrElse {
                    amount: 1,
                    otherwise: Box::new(Effect::SacrificeSource),
                },
            },
        ],
        ..Default::default()
    }
}

/// Exploration — {G} Enchantment (Urza's Saga reprint). "You may play
/// an additional land on each of your turns." Single static grant of
/// `ExtraLandPerTurn`. The per-turn land cap is checked by
/// [`GameState::can_player_play_land`] (CR 305.2a), which sums every
/// `ExtraLandPerTurn` static effect controlled by the active player.
/// Stacks multiplicatively: two Explorations → three lands per turn.
pub fn exploration() -> CardDefinition {
    CardDefinition {
        name: "Exploration",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        ..Default::default()
    }
}

/// Ghostly Prison — {2}{W} Enchantment. Creatures can't attack you unless
/// their controller pays {2} for each such attacker (CR 508.1g attack tax).
pub fn ghostly_prison() -> CardDefinition {
    CardDefinition {
        name: "Ghostly Prison",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures can't attack you unless their controller pays {2} for each.",
            effect: StaticEffect::AttackTaxToController { amount: Value::Const(2), protect_planeswalkers: false },
        }],
        ..Default::default()
    }
}

/// Propaganda — {2}{U} Enchantment. Blue twin of Ghostly Prison.
pub fn propaganda() -> CardDefinition {
    CardDefinition {
        name: "Propaganda",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures can't attack you unless their controller pays {2} for each.",
            effect: StaticEffect::AttackTaxToController { amount: Value::Const(2), protect_planeswalkers: false },
        }],
        ..Default::default()
    }
}

/// Sphere of Safety — {3}{W} Enchantment. "Creatures can't attack you or a
/// planeswalker you control unless their controller pays {X} for each of those
/// creatures, where X is the number of enchantments you control." The dynamic
/// tax uses `Value::count(enchantments you control)` (which counts Sphere
/// itself), evaluated against the defending player in `declare_attackers`.
pub fn sphere_of_safety() -> CardDefinition {
    CardDefinition {
        name: "Sphere of Safety",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures can't attack you or a planeswalker you control unless their controller pays {X} for each, where X is the number of enchantments you control.",
            effect: StaticEffect::AttackTaxToController {
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Enchantment.and(SelectionRequirement::ControlledByYou),
                )),
                protect_planeswalkers: true,
            },
        }],
        ..Default::default()
    }
}

/// Beastmaster Ascension — {1}{G} Enchantment. "Whenever a creature you control
/// attacks, put a quest counter on Beastmaster Ascension. As long as it has
/// seven or more quest counters, creatures you control get +5/+5." The anthem
/// is a `StaticEffect::PumpTeamIf` gated on the quest-counter threshold.
pub fn beastmaster_ascension() -> CardDefinition {
    CardDefinition {
        name: "Beastmaster Ascension",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(crate::effect::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: crate::card::CounterType::Quest,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "As long as this has seven or more quest counters, creatures you control get +5/+5.",
            effect: StaticEffect::PumpTeamIf {
                condition: crate::effect::Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: crate::card::CounterType::Quest,
                    },
                    Value::Const(7),
                ),
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: 5,
                toughness: 5,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Aura Shards — {G}{W} Enchantment. "Whenever a creature you control enters,
/// you may destroy target artifact or enchantment." The optional clause is
/// collapsed to a mandatory destroy-if-a-legal-target-exists (matching
/// Reclamation Sage's ETB), and auto-targeting prefers an opponent's permanent.
pub fn aura_shards() -> CardDefinition {
    CardDefinition {
        name: "Aura Shards",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(crate::effect::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Aura of Silence — {1}{W}{W} Enchantment. "Artifact and enchantment spells
/// cost {2} more to cast." (Printed: only opponents' spells; modeled as an
/// all-players `AdditionalCost` tax.) "Sacrifice this: Destroy target artifact
/// or enchantment."
pub fn aura_of_silence() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Aura of Silence",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Artifact and enchantment spells cost {2} more to cast.",
            effect: StaticEffect::AdditionalCost {
                filter: SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                amount: 2,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Return to Dust — {2}{W}{W} Sorcery. "Exile target artifact or enchantment."
/// (The "if you cast this at sorcery speed, you may exile up to one additional"
/// rider is simplified to a single target.)
pub fn return_to_dust() -> CardDefinition {
    CardDefinition {
        name: "Return to Dust",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
            ),
        },
        ..Default::default()
    }
}

/// Rhystic Study — {2}{U} Enchantment. "Whenever an opponent casts a spell, you
/// may draw a card unless that player pays {1}." The caster is asked to pay {1}
/// (`UnlessPlayerPays`, `PlayerRef::Triggerer`); if they decline/can't, you draw.
pub fn rhystic_study() -> CardDefinition {
    CardDefinition {
        name: "Rhystic Study",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::Triggerer,
                cost: crate::card::WardCost::generic(1),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            },
        }],
        ..Default::default()
    }
}

/// Mystic Remora — {U} Enchantment. Cumulative upkeep {1} (CR 702.24).
/// "Whenever an opponent casts a noncreature spell, you may draw a card unless
/// that player pays {4}." The caster is asked to pay {4} (`UnlessPlayerPays`);
/// if they decline/can't, you draw.
pub fn mystic_remora() -> CardDefinition {
    use crate::card::{CumulativeUpkeepCost, Keyword};
    CardDefinition {
        name: "Mystic Remora",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(crate::effect::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Noncreature,
                }),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::Triggerer,
                cost: crate::card::WardCost::generic(4),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            },
        }],
        ..Default::default()
    }
}

/// Smothering Tithe — {3}{W} Enchantment. "Whenever an opponent draws a card,
/// that player may pay {2}. If they don't, you create a Treasure token."
/// (`UnlessPlayerPays`, `PlayerRef::Triggerer` = the drawing opponent.)
pub fn smothering_tithe() -> CardDefinition {
    CardDefinition {
        name: "Smothering Tithe",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::Triggerer,
                cost: crate::card::WardCost::generic(2),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Mana Drain — {U}{U} Instant. "Counter target spell." (The "add {C} for each
/// of its mana value at your next precombat main phase" ritual rider is
/// omitted.)
pub fn mana_drain() -> CardDefinition {
    CardDefinition {
        name: "Mana Drain",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell { what: target_filtered(SelectionRequirement::Any) },
        ..Default::default()
    }
}

/// Fierce Guardianship — {2}{U} Instant. "Counter target noncreature spell."
/// (The "if you control your commander, this costs {0}" alt-cost is omitted.)
pub fn fierce_guardianship() -> CardDefinition {
    CardDefinition {
        name: "Fierce Guardianship",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell { what: target_filtered(SelectionRequirement::Noncreature) },
        ..Default::default()
    }
}

/// Deflecting Swat — {2}{R} Instant. "Counter target spell." (Printed: counter
/// target spell or ability and you may choose new targets; the new-targets
/// rider and the free-with-commander alt-cost are omitted.)
pub fn deflecting_swat() -> CardDefinition {
    CardDefinition {
        name: "Deflecting Swat",
        cost: cost(&[generic(2), crate::mana::r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell { what: target_filtered(SelectionRequirement::Any) },
        ..Default::default()
    }
}

/// Collective Restraint — {3}{U} Enchantment. Domain — creatures can't
/// attack you unless their controller pays {X} per attacker, X = the number
/// of basic land types among lands you control.
pub fn collective_restraint() -> CardDefinition {
    CardDefinition {
        name: "Collective Restraint",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Domain — Creatures can't attack you unless their controller pays {X} for each creature they control that's attacking you, where X is the number of basic land types among lands you control.",
            effect: StaticEffect::AttackTaxToController {
                amount: Value::DomainCount(PlayerRef::You),
                protect_planeswalkers: false,
            },
        }],
        ..Default::default()
    }
}

/// Luminarch Ascension — {1}{W} Enchantment. At the beginning of each
/// opponent's end step, if you didn't lose life this turn, you may put a
/// quest counter on it. {1}{W}: Create a 4/4 white Angel with flying —
/// activate only with four or more quest counters.
pub fn luminarch_ascension() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType, CreatureType, Keyword, Predicate, Subtypes, TokenDefinition, Value as V};
    use crate::mana::{Color, w};
    CardDefinition {
        name: "Luminarch Ascension",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::All(vec![
                Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You))),
                Predicate::Not(Box::new(Predicate::PlayerLostLifeThisTurn {
                    who: PlayerRef::You,
                })),
            ])),
            effect: Effect::MayDo {
                description: "Put a quest counter on Luminarch Ascension".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Quest,
                    amount: V::Const(1),
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            condition: Some(Predicate::ValueAtLeast(
                V::CountersOn { what: Box::new(Selector::This), kind: CounterType::Quest },
                V::Const(4),
            )),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: V::Const(1),
                definition: TokenDefinition {
                    name: "Angel".into(),
                    power: 4,
                    toughness: 4,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    keywords: vec![Keyword::Flying],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Angel],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Psychic Corrosion — {2}{U} Enchantment. Whenever you draw a card, each
/// opponent mills two cards.
pub fn psychic_corrosion() -> CardDefinition {
    CardDefinition {
        name: "Psychic Corrosion",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Drowned Secrets — {1}{U} Enchantment. Whenever you cast a blue spell,
/// target player mills two cards.
pub fn drowned_secrets() -> CardDefinition {
    use crate::card::Predicate;
    use crate::mana::Color;
    CardDefinition {
        name: "Drowned Secrets",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasColor(Color::Blue),
                },
            ),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Underworld Connections — {1}{B}{B} Enchantment — Aura. Enchant land;
/// enchanted land has "{T}, Pay 1 life: Draw a card."
pub fn underworld_connections() -> CardDefinition {
    use crate::card::{ActivatedAbility, EnchantmentSubtype};
    use crate::mana::b;
    CardDefinition {
        name: "Underworld Connections",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: crate::card::Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Land,
            },
        },
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}, Pay 1 life: Draw a card.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    life_cost: 1,
                    effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}
