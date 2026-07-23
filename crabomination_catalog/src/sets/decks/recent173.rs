//! More Aetherdrift (DFT) gap cards on existing primitives: a Pilot-minting
//! Aura, a control-swap sorcery, an affinity draw spell, and a
//! reveal-hand-and-exile disruption spell. Tests in `tests/recent173.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Selector};
use crate::mana::{b, cost, generic, r, u, w};

/// The 1/1 colorless Pilot token with the +2 crew/saddle bonus.
fn boosted_pilot_token() -> TokenDefinition {
    TokenDefinition {
        name: "Pilot".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pilot], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This token saddles Mounts and crews Vehicles as though its power were 2 greater.",
            effect: StaticEffect::CrewSaddlePowerBonus { applies_to: Selector::This, amount: 2 },
        }],
        ..Default::default()
    }
}

/// Roadside Assistance — {2}{W} Aura. Enchant creature or Vehicle. ETB: create a
/// boosted Pilot. Enchanted permanent gets +1/+1 and has lifelink.
pub fn roadside_assistance() -> CardDefinition {
    CardDefinition {
        name: "Roadside Assistance",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature.or(R::HasArtifactSubtype(crate::card::ArtifactSubtype::Vehicle)),
            },
        },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Lifelink],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: boosted_pilot_token(),
        })],
        ..Default::default()
    }
}

/// Cloudspire Coordinator — {R}{W} 3/1 Human Pilot. ETB: scry 2. {T}: create X
/// 1/1 colorless Pilot tokens, X = the number of Mounts and/or Vehicles that
/// entered under your control this turn (each token crews/saddles as though its
/// power were 2 greater).
pub fn cloudspire_coordinator() -> CardDefinition {
    CardDefinition {
        name: "Cloudspire Coordinator",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::MountsVehiclesEnteredThisTurn(PlayerRef::You),
                definition: boosted_pilot_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Trade the Helm — {4}{U} Sorcery. Exchange control of target artifact or
/// creature you control and one an opponent controls. Cycling {2}.
pub fn trade_the_helm() -> CardDefinition {
    CardDefinition {
        name: "Trade the Helm",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::ExchangeControl {
            a: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature
                    .or(R::Artifact)
                    .and(R::ControlledByYou),
            },
            b: Selector::TargetFiltered {
                slot: 1,
                filter: R::Creature
                    .or(R::Artifact)
                    .and(R::ControlledByOpponent),
            },
        },
        ..Default::default()
    }
}

/// Voyage Home — {5}{W}{U} Sorcery. Affinity for artifacts. You draw three cards
/// and gain 3 life.
pub fn voyage_home() -> CardDefinition {
    CardDefinition {
        name: "Voyage Home",
        cost: cost(&[generic(5), w(), u()]),
        card_types: vec![CardType::Sorcery],
        affinity_filter: Some(R::Artifact),
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Aggressive Negotiations — {2}{B} Sorcery. Target opponent reveals their hand;
/// you exile a nonland card from it. Put a +1/+1 counter on up to one target
/// creature you control. (Reveal targeting approximated as each opponent.)
pub fn aggressive_negotiations() -> CardDefinition {
    CardDefinition {
        name: "Aggressive Negotiations",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileChosenFromHand {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Nonland,
                link_to_source: false,
                face_down: false,
            },
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByYou),
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        ]),
        ..Default::default()
    }
}
