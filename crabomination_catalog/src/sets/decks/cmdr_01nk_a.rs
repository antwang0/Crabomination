//! Commander: the 2026-10-20 (`01NkAE5B`) one-gap pair closers. Tests in
//! `tests/recent_b/cmdr_01nk_a.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{discard, draw, etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, Value, ZoneRef};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

/// Blossombind — {1}{U} Aura. When it enters, tap enchanted creature.
/// Enchanted creature can't become untapped and can't have counters put on it.
pub fn blossombind() -> CardDefinition {
    CardDefinition {
        name: "Blossombind",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::Tap { what: enchanted() })],
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature can't become untapped.",
                effect: StaticEffect::CantBecomeUntapped { applies_to: enchanted() },
            },
            StaticAbility {
                description: "Enchanted creature can't have counters put on it.",
                effect: StaticEffect::CantHaveCountersPutOn { applies_to: enchanted() },
            },
        ],
        ..Default::default()
    }
}

/// Purity — {3}{W}{W}{W} 6/6 flying Elemental Incarnation. Noncombat damage
/// to you is prevented and you gain that much life; put into a graveyard from
/// anywhere, it shuffles into its owner's library.
pub fn purity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If noncombat damage would be dealt to you, prevent that damage. You gain life equal to the damage prevented this way.",
            effect: StaticEffect::PreventNoncombatDamageToYouGainLife,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
            effect: Effect::ShuffleSelfIntoLibrary,
        }],
        ..creature(
            "Purity",
            cost(&[generic(3), w(), w(), w()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            6,
            6,
        )
    }
}

/// Creeping Trailblazer — {R}{G} 2/2 Elemental. Other Elementals you control
/// get +1/+0. {2}{R}{G}: +1/+1 until end of turn for each Elemental you control.
pub fn creeping_trailblazer() -> CardDefinition {
    let elementals = || {
        Value::PermanentCountControlledByMatching(PlayerRef::You, R::HasCreatureType(CreatureType::Elemental))
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Elementals you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Elemental).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r(), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: elementals(),
                toughness: elementals(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Creeping Trailblazer", cost(&[r(), g()]), vec![CreatureType::Elemental], 2, 2)
    }
}

/// Mantle of Leadership — {1}{W} Aura with flash. Whenever a creature enters,
/// enchanted creature gets +2/+2 until end of turn.
pub fn mantle_of_leadership() -> CardDefinition {
    CardDefinition {
        name: "Mantle of Leadership",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::PumpPT {
                what: enchanted(),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Timeline Inquiry — {3}{U} instant, teamwork 2. Draw three cards, then
/// discard a card unless it was cast using teamwork.
pub fn timeline_inquiry() -> CardDefinition {
    CardDefinition {
        name: "Timeline Inquiry",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower { filter: R::Creature.and(R::ControlledByYou), power: 2 }),
        effect: Effect::Seq(vec![
            draw(3),
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::Noop),
                else_: Box::new(discard(Selector::You, 1, false)),
            },
        ]),
        ..Default::default()
    }
}

/// Stormfront Riders — {4}{W} 4/3 flying Human Soldier. When it enters,
/// return two creatures you control to their owner's hand. Whenever a
/// creature is returned to your hand from the battlefield, create a 1/1
/// white Soldier. (A Riders returning itself is seen only while another
/// copy is on the battlefield — leaves-the-battlefield look-back for this
/// event kind is not modeled.)
pub fn stormfront_riders() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    let bounce = || Effect::ReturnOneYouControl { filter: R::Creature, keep_best: false };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![bounce(), bounce()])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentReturnedToHand, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(soldier) },
            },
        ],
        ..creature(
            "Stormfront Riders",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            3,
        )
    }
}

/// Impelled Giant — {4}{R}{R} 3/3 trample Giant Warrior. Tap an untapped red
/// creature you control other than this creature: it gets +X/+0 until end of
/// turn, where X is the tapped creature's power.
pub fn impelled_giant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(
                R::Creature.and(R::HasColor(Color::Red)).and(R::ControlledByYou).and(R::OtherThanSource),
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::TappedForCostPower,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Impelled Giant",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Giant, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Doubling Chant — {5}{G} sorcery. For each creature you control, you may
/// search your library for a creature card with the same name, onto the
/// battlefield, then shuffle.
pub fn doubling_chant() -> CardDefinition {
    CardDefinition {
        name: "Doubling Chant",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            body: Box::new(Effect::SearchSameNameToBattlefield { who: PlayerRef::You, what: Selector::TriggerSource }),
        },
        ..Default::default()
    }
}

/// Dichotomancy — {7}{U}{U} sorcery, suspend 3—{1}{U}{U}. For each tapped
/// nonland permanent target opponent controls, search their library for a
/// card with the same name and put it onto the battlefield under your
/// control; they shuffle.
pub fn dichotomancy() -> CardDefinition {
    CardDefinition {
        name: "Dichotomancy",
        cost: cost(&[generic(7), u(), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Suspend(3, cost(&[generic(1), u(), u()]))],
        effect: Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::ForEach {
                selector: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Tapped.and(R::Nonland) },
                body: Box::new(Effect::SearchOpponentLibraryForSameName { what: Selector::TriggerSource }),
            }),
        },
        ..Default::default()
    }
}

/// Sphere of Annihilation — {X}{B} artifact; enters with X void counters. At
/// the beginning of your upkeep, exile it, every creature and planeswalker
/// with mana value at most its void counters, and every such card in a
/// graveyard.
pub fn sphere_of_annihilation() -> CardDefinition {
    // Per object, against the live counter count (the Sphere goes last).
    let exile_small = |zone: Selector| Effect::ForEach {
        selector: zone,
        body: Box::new(Effect::If {
            cond: Predicate::ValueAtMost(
                Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Void },
            ),
            then: Box::new(Effect::Exile { what: Selector::TriggerSource }),
            else_: Box::new(Effect::Noop),
        }),
    };
    let doomed = || R::Creature.or(R::Planeswalker);
    CardDefinition {
        name: "Sphere of Annihilation",
        cost: cost(&[x(), b()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Void, Value::XFromCost)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                exile_small(Selector::EachPermanent(doomed())),
                exile_small(Selector::EachMatching { zone: ZoneRef::Graveyard(PlayerRef::EachPlayer), filter: doomed() }),
                Effect::Exile { what: Selector::This },
            ]),
        }],
        ..Default::default()
    }
}

/// Tarnation Vista — land. Enters tapped; as it enters, choose a color. {T}:
/// add one mana of the chosen color. {1}, {T}: for each color among
/// monocolored permanents you control, add one mana of that color.
pub fn tarnation_vista() -> CardDefinition {
    CardDefinition {
        name: "Tarnation Vista",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::ChosenColorOfSource },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OneOfEachColorAmongYourMonocoloredPermanents,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
