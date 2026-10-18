//! Commander: 2026-10-18 (`019Y6J4R`) closers, twentieth batch. Tests in
//! `tests/recent_b/cmdr_019y_t.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{ActivatedAbility, Duration, Effect, PlayerRef, Predicate, Selector, StaticEffect, Value};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, u};

/// Dawnhand Dissident — {B} 1/2 Elf Warlock. {T}, Blight 1: surveil 1.
/// {T}, Blight 2: exile target card from a graveyard (linked to this). During
/// your turn, you may cast creature spells from among cards you own exiled
/// with it by removing three counters from among creatures you control in
/// addition to paying their other costs.
pub fn dawnhand_dissident() -> CardDefinition {
    CardDefinition {
        name: "Dawnhand Dissident",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Warlock], ..Default::default() },
        power: 1,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                blight_cost: 1,
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                blight_cost: 2,
                effect: Effect::ExileWithSource { what: target_filtered(R::InGraveyard) },
                ..Default::default()
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "During your turn, you may cast creature spells from among cards you own exiled with this creature by removing three counters from among creatures you control in addition to paying their other costs.",
            effect: StaticEffect::CastLinkedExiledCreaturesForCounters { counters: 3 },
        }],
        ..Default::default()
    }
}

/// Cryptic Spires — Land. As you create your deck, circle two colors. Enters
/// tapped; {T}: add one mana of either circled color
/// (`ManaPayload::CircledColors` — read off the commander identity).
pub fn cryptic_spires() -> CardDefinition {
    CardDefinition {
        name: "Cryptic Spires",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: crate::effect::ManaPayload::CircledColors },
            ..Default::default()
        }],
        static_abilities: vec![crate::sets::enters_tapped()],
        ..Default::default()
    }
}

/// Absorbing Man — {1}{G}{U} legendary 4/4 vigilance Human Villain. At the
/// beginning of your first main phase, until your next turn, he becomes a
/// copy of up to one target artifact, non-Aura enchantment, or land, except
/// his name is Absorbing Man, he's a legendary 4/4 Human Villain creature in
/// addition to his other types, and he has vigilance (CR 707.9b).
pub fn absorbing_man() -> CardDefinition {
    CardDefinition {
        name: "Absorbing Man",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Villain], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Artifact
                    .or(R::Enchantment.and(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura).negate()))
                    .or(R::Land)
                    .and(R::OtherThanSource),
                effect: Box::new(Effect::Seq(vec![
                    Effect::BecomeCopyOfFor {
                        what: Selector::This,
                        source: Selector::Target(0),
                        duration: Duration::UntilNextTurn,
                        non_legendary: false,
                    },
                    Effect::AmendCopy {
                        what: Selector::This,
                        keep_activated: vec![],
                        keep_triggered: vec![],
                        pt: Some((4, 4)),
                        keep_name: true,
                        legendary: true,
                        keywords: vec![Keyword::Vigilance],
                        card_types: vec![CardType::Creature],
                        creature_types: vec![CreatureType::Human, CreatureType::Villain],
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Hurkyl, Master Wizard — {1}{U}{U} legendary 2/4 Human Wizard Advisor. At
/// the beginning of your end step, if you've cast a noncreature spell this
/// turn, reveal the top five cards of your library; for each card type among
/// noncreature spells you've cast this turn, you may put a card of that type
/// from among them into your hand. The rest go on the bottom in a random order.
pub fn hurkyl_master_wizard() -> CardDefinition {
    CardDefinition {
        name: "Hurkyl, Master Wizard",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                Predicate::NoncreatureSpellsCastThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE },
            ),
            effect: Effect::RevealTopTakeOnePerCastType { who: PlayerRef::You, count: Value::Const(5) },
        }],
        ..Default::default()
    }
}
