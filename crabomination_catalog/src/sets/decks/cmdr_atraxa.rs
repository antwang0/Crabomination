//! Commander: the cards the **Breed Lethality** precon (C16, Atraxa,
//! Praetors' Voice) needed beyond what the catalog had (Champion of Lambholt
//! is Token Triumph's, `cmdr_emmara.rs`; Dreadship Reef is Devour for
//! Power's, `cmdr_mimeoplasm.rs`). Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Duneblast** — the survivor is the chooser's pick among all creatures,
//!   and one always survives when any exist ("up to one" never picks none).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnterMode, Keyword,
    LandType, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, u, w};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn plus_one(what: Selector) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }
}

fn another_of_yours_enters() -> EventSpec {
    EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
        Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: R::Creature.and(R::OtherThanSource),
        },
    )
}

/// Cauldron of Souls — {T}: any number of creatures gain persist until end
/// of turn.
pub fn cauldron_of_souls() -> CardDefinition {
    CardDefinition {
        name: "Cauldron of Souls",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::ApplyToTargets {
                max_targets: 16,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Persist,
                    duration: Duration::EndOfTurn,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Citadel Siege — Khans: two +1/+1 counters at the start of your combat;
/// Dragons: tap a creature of each opponent at the start of their combat.
pub fn citadel_siege() -> CardDefinition {
    CardDefinition {
        name: "Citadel Siege",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        enter_modes: Some(vec![
            EnterMode {
                label: "Khans",
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::StepBegins(TurnStep::BeginCombat),
                        EventScope::YourControl,
                    ),
                    effect: Effect::AddCounter {
                        what: target_filtered(R::Creature.and(R::ControlledByYou)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    },
                }],
                ..Default::default()
            },
            EnterMode {
                label: "Dragons",
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::StepBegins(TurnStep::BeginCombat),
                        EventScope::OpponentControl,
                    ),
                    effect: Effect::Tap {
                        what: target_filtered(R::Creature.and(R::ControlledByActivePlayer)),
                    },
                }],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Deepglow Skate — double each kind of counter on any number of permanents.
pub fn deepglow_skate() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 16,
            min_targets: 0,
            filter: R::Permanent,
            effect: Box::new(Effect::DoubleAllCountersOn { what: Selector::Target(0) }),
        })],
        ..creature("Deepglow Skate", cost(&[generic(4), u()]), vec![CreatureType::Fish], 3, 3)
    }
}

/// Duelist's Heritage — whenever creatures attack, an attacker may gain
/// double strike until end of turn.
pub fn duelists_heritage() -> CardDefinition {
    CardDefinition {
        name: "Duelist's Heritage",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Duneblast — choose up to one creature; destroy the rest.
pub fn duneblast() -> CardDefinition {
    CardDefinition {
        name: "Duneblast",
        cost: cost(&[generic(4), w(), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseOneAmong {
            what: Selector::EachPermanent(R::Creature),
            chooser: PlayerRef::You,
            chosen: Box::new(Effect::Noop),
            other: Box::new(Effect::Destroy { what: Selector::SeparatedPile { chosen: false } }),
        },
        ..Default::default()
    }
}

/// Enduring Scalelord — flying; +1/+1 counters landing on another creature
/// of yours may add one to it.
pub fn enduring_scalelord() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::OtherThanSource),
            }),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on Enduring Scalelord?".into(),
                body: Box::new(plus_one(Selector::This)),
            },
        }],
        ..creature(
            "Enduring Scalelord",
            cost(&[generic(4), g(), w()]),
            vec![CreatureType::Dragon],
            4,
            4,
        )
    }
}

/// Festercreep — enters with a +1/+1 counter; {1}{B}, remove it: every other
/// creature gets -1/-1 until end of turn.
pub fn festercreep() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Festercreep", cost(&[generic(1), b()]), vec![CreatureType::Elemental], 0, 0)
    }
}

/// Ikra Shidiqi, the Usurper — menace; your creatures' combat damage to a
/// player gains you life equal to their toughness; partner.
pub fn ikra_shidiqi_the_usurper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..legendary(creature(
            "Ikra Shidiqi, the Usurper",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Snake, CreatureType::Wizard],
            3,
            7,
        ))
    }
}

/// Ishai, Ojutai Dragonspeaker — flying; an opponent's spell adds a +1/+1
/// counter; partner.
pub fn ishai_ojutai_dragonspeaker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: plus_one(Selector::This),
        }],
        ..legendary(creature(
            "Ishai, Ojutai Dragonspeaker",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Bird, CreatureType::Monk],
            1,
            1,
        ))
    }
}

/// Juniper Order Ranger — another creature of yours entering gets a +1/+1
/// counter, and so does the Ranger.
pub fn juniper_order_ranger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: another_of_yours_enters(),
            effect: Effect::Seq(vec![plus_one(Selector::TriggerSource), plus_one(Selector::This)]),
        }],
        ..creature(
            "Juniper Order Ranger",
            cost(&[generic(3), g(), w()]),
            vec![CreatureType::Human, CreatureType::Knight, CreatureType::Ranger],
            2,
            4,
        )
    }
}

/// Manifold Insights — reveal ten; each opponent picks you a different
/// nonland card; the rest go to the bottom at random.
pub fn manifold_insights() -> CardDefinition {
    CardDefinition {
        name: "Manifold Insights",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RevealTopEachOpponentChoosesToHand {
            count: Value::Const(10),
            pick_filter: R::Nonland,
        },
        ..Default::default()
    }
}

/// Mirrorweave — every other creature becomes a copy of a nonlegendary
/// creature until end of turn.
pub fn mirrorweave() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    CardDefinition {
        name: "Mirrorweave",
        cost: cost(&[generic(2), wu(), wu()]),
        card_types: vec![CardType::Instant],
        effect: Effect::BecomeCopyOfFor {
            what: Selector::EachPermanent(R::Creature),
            source: target_filtered(
                R::Creature.and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary)))),
            ),
            duration: Duration::EndOfTurn,
            non_legendary: false,
        },
        ..Default::default()
    }
}

/// Murmuring Bosk — a Forest that enters tapped unless you reveal a
/// Treefolk; {T}: {W} or {B}, 1 damage to you.
pub fn murmuring_bosk() -> CardDefinition {
    CardDefinition {
        name: "Murmuring Bosk",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Forest], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "As this land enters, you may reveal a Treefolk card from your hand. If you don't, this land enters tapped.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorExists(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Hand,
                    filter: R::HasCreatureType(CreatureType::Treefolk),
                }),
            },
        }],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Green),
            crate::sets::pain_tap(Color::White),
            crate::sets::pain_tap(Color::Black),
        ],
        ..Default::default()
    }
}

/// Reyhan, Last of the Abzan — enters with three +1/+1 counters; a creature
/// of yours dying with +1/+1 counters may pass that many to a creature;
/// partner.
pub fn reyhan_last_of_the_abzan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Partner],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::WithCounter(CounterType::PlusOnePlusOne),
                },
            ),
            effect: Effect::MayDo {
                description: "Move its +1/+1 counters onto a creature?".into(),
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CountersOn {
                        what: Box::new(Selector::TriggerSource),
                        kind: CounterType::PlusOnePlusOne,
                    },
                }),
            },
        }],
        ..legendary(creature(
            "Reyhan, Last of the Abzan",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            0,
            0,
        ))
    }
}
