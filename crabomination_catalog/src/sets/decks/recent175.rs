//! Aetherdrift (DFT) gap batch: batched-discard burn (Magmakin), a max-speed
//! "each player without max speed" enchantment (Outpace Oblivion), an
//! attacked-by debuff (Sabotage Strategist), a copy-an-artifact-or-creature
//! Shapeshifter (Waxen Shapethief), mill+conditional-destroy (Quag Feast), a
//! modal fight/destroy (Plow Through), a blink+board-wipe (Explosive Getaway),
//! and a Start-your-engines Aura (Lightwheel Enhancements). Tests in
//! `crabomination/src/tests/recent175.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EntersAsCopy, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Magmakin Artillerist — {2}{R} 1/4 Elemental Pirate. Whenever you discard one
/// or more cards, deal that much damage to each opponent. Cycling {1}{R}. When
/// you cycle this card, it deals 1 damage to each opponent.
pub fn magmakin_artillerist() -> CardDefinition {
    let bolt_opponents = |amount: Value| Effect::DealDamage {
        to: Selector::Player(PlayerRef::EachOpponent),
        amount,
    };
    CardDefinition {
        name: "Magmakin Artillerist",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Cycling(cost(&[generic(1), r()]))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DiscardedOneOrMore, EventScope::YourControl),
                effect: bolt_opponents(Value::TriggerEventAmount),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
                effect: bolt_opponents(Value::ONE),
            },
        ],
        ..Default::default()
    }
}

/// Outpace Oblivion — {2}{R} Enchantment. Start your engines! ETB: deal 5 damage
/// to up to one target creature or planeswalker. {2}, Sacrifice this: deal 2
/// damage to each player who doesn't have max speed.
pub fn outpace_oblivion() -> CardDefinition {
    CardDefinition {
        name: "Outpace Oblivion",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            filter: R::Creature.or(R::Planeswalker),
            effect: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(5) }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayerWithoutMaxSpeed),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sabotage Strategist — {2}{U}{U} 2/2 Vedalken Ranger. Flying, vigilance.
/// Whenever one or more creatures attack you, those creatures get -1/-0 until
/// end of turn. Exhaust — {5}{U}{U}: put three +1/+1 counters on this.
/// (Attacks on a planeswalker you control also fire it — a slight over-fire.)
pub fn sabotage_strategist() -> CardDefinition {
    CardDefinition {
        name: "Sabotage Strategist",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Ranger],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(-1),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u(), u()]),
            exhaust: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Waxen Shapethief — {3}{U} 0/0 Shapeshifter with flash. Enters as a copy of an
/// artifact or creature you control (may). Cycling {2}.
pub fn waxen_shapethief() -> CardDefinition {
    CardDefinition {
        name: "Waxen Shapethief",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
        keywords: vec![Keyword::Flash, Keyword::Cycling(cost(&[generic(2)]))],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature.or(R::Artifact).and(R::ControlledByYou),
            extra_creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Quag Feast — {1}{B} Sorcery. Choose target creature, planeswalker, or
/// Vehicle. Mill two, then destroy it if its mana value ≤ cards in your
/// graveyard.
pub fn quag_feast() -> CardDefinition {
    let target = Selector::TargetFiltered {
        slot: 0,
        filter: R::Creature.or(R::Planeswalker).or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
    };
    CardDefinition {
        name: "Quag Feast",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(2) },
            Effect::If {
                cond: Predicate::ValueAtMost(
                    Value::ManaValueOf(Box::new(target.clone())),
                    Value::GraveyardSizeOf(PlayerRef::You),
                ),
                then: Box::new(Effect::Destroy { what: target.clone() }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Plow Through — {G} Sorcery. Choose one — your creature fights an opponent's
/// creature; or destroy target Vehicle.
pub fn plow_through() -> CardDefinition {
    CardDefinition {
        name: "Plow Through",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Fight {
                attacker: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            },
            Effect::Destroy {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::HasArtifactSubtype(ArtifactSubtype::Vehicle),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Explosive Getaway — {3}{R}{W} Sorcery. Exile up to one target artifact or
/// creature, returning it at the next end step; deal 4 damage to each creature.
pub fn explosive_getaway() -> CardDefinition {
    CardDefinition {
        name: "Explosive Getaway",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                filter: R::Artifact.or(R::Creature),
                effect: Box::new(Effect::ExileReturnNextEndStep { what: Selector::Target(0) }),
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(R::Creature),
                body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(4) }),
            },
        ]),
        ..Default::default()
    }
}

/// Lightwheel Enhancements — {W} Aura. Start your engines! Enchant creature or
/// Vehicle; it gets +1/+1 and has vigilance. (Max-speed graveyard-cast dropped.)
pub fn lightwheel_enhancements() -> CardDefinition {
    CardDefinition {
        name: "Lightwheel Enhancements",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::StartYourEngines],
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
            },
        },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Vigilance],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Thopter Fabricator — {2}{U} Artifact — Vehicle 4/4. Flying. Whenever you draw
/// your second card each turn, create a 1/1 colorless Thopter artifact creature
/// token with flying. Crew 2.
pub fn thopter_fabricator() -> CardDefinition {
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Thopter Fabricator",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::Triggerer, n: 2 })
                .once_per_turn(),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: thopter },
        }],
        ..Default::default()
    }
}
