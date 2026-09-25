//! Commander: the cards the **Veloci-Ramp-Tor** precon (LCC, Pantlaza,
//! Sun-Favored) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_pantlaza.rs`.
//!
//! Residuals (each also on its card):
//! - **Sunfrill Imitator** — the copy takes the copied Dinosaur's name, not
//!   "Sunfrill Imitator" (only a legendary copy target tells the difference).
//! - **Wrathful Raptors** — a Dinosaur's damage that kills the Raptors
//!   themselves in the same event doesn't trigger them.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType, EventKind,
    EventScope, EventSpec, ExileReturnZone, Keyword, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, myriad, on_attack, on_attack_ping_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r, w, x};
use crate::sets::tap_add_any_color;
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

fn dino(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    creature(name, mana, vec![CreatureType::Dinosaur], p, t)
}

fn dinosaur() -> R {
    R::HasCreatureType(CreatureType::Dinosaur)
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn eot_pump(what: Selector, p: Value, t: Value) -> Effect {
    Effect::PumpPT { what, power: p, toughness: t, duration: Duration::EndOfTurn }
}

/// Atzocan Seer — {T}: one mana of any color; sacrifice it: return target
/// Dinosaur card from your graveyard to your hand.
pub fn atzocan_seer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_add_any_color(),
            ActivatedAbility {
                sac_cost: true,
                effect: Effect::Move {
                    what: target_filtered(dinosaur().from_your_graveyard()),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Atzocan Seer",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Human, CreatureType::Druid],
            2,
            3,
        )
    }
}

/// Bellowing Aegisaur — enrage: a +1/+1 counter on each other creature you
/// control.
pub fn bellowing_aegisaur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(yours(R::Creature).and(R::OtherThanSource)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..dino("Bellowing Aegisaur", cost(&[generic(5), w()]), 3, 5)
    }
}

/// Bronzebeak Foragers — ETB: for each opponent, exile up to one target
/// nonland permanent they control until this leaves; {X}{W}: put a card
/// with mana value X exiled with it into its owner's graveyard, gain X.
pub fn bronzebeak_foragers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Permanent.and(R::Nonland).and(R::ControlledByOpponent),
                effect: Box::new(Effect::ExileUntilSourceLeaves {
                    what: Selector::Target(0),
                    return_to: ExileReturnZone::Battlefield,
                }),
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), w()]),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::ExiledWithSource.and(R::ManaValueExactlyXFromCost)),
                    to: ZoneDest::Graveyard,
                },
                Effect::GainLife { who: Selector::You, amount: Value::XFromCost },
            ]),
            ..Default::default()
        }],
        ..dino("Bronzebeak Foragers", cost(&[generic(3), w()]), 3, 4)
    }
}

/// Deathgorge Scavenger — enters or attacks: you may exile target card from
/// a graveyard; a creature card gains you 2 life, a noncreature card gives
/// it +1/+1 until end of turn.
pub fn deathgorge_scavenger() -> CardDefinition {
    let scavenge = || Effect::MayDo {
        description: "Exile target card from a graveyard".into(),
        body: Box::new(Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::Any.from_any_graveyard()), to: ZoneDest::Exile },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Creature },
                then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
                else_: Box::new(eot_pump(Selector::This, Value::ONE, Value::ONE)),
            },
        ])),
    };
    CardDefinition {
        triggered_abilities: vec![etb(scavenge()), on_attack(scavenge())],
        ..dino("Deathgorge Scavenger", cost(&[generic(2), g()]), 3, 2)
    }
}

/// Descendants' Path — your upkeep: reveal the top card; a creature card
/// sharing a creature type with a creature you control may be cast free,
/// else it goes to the bottom.
pub fn descendants_path() -> CardDefinition {
    CardDefinition {
        name: "Descendants' Path",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::RevealTopMayCastOneFree {
                count: Value::ONE,
                max_mv: Value::Const(i32::MAX),
                filter: Some(R::Creature.and(R::SharesCreatureTypeWithCreatureYouControl)),
            },
        }],
        ..Default::default()
    }
}

/// Drover of the Mighty — +2/+2 while you control a Dinosaur; {T}: one mana
/// of any color.
pub fn drover_of_the_mighty() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +2/+2 as long as you control a Dinosaur.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(yours(dinosaur())),
                    n: Value::ONE,
                },
                power: 2,
                toughness: 2,
                keywords: vec![],
            },
        }],
        activated_abilities: vec![tap_add_any_color()],
        ..creature(
            "Drover of the Mighty",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// From the Rubble — as it enters, choose a creature type; your end step:
/// return target creature card of that type from your graveyard to the
/// battlefield with a finality counter.
pub fn from_the_rubble() -> CardDefinition {
    CardDefinition {
        name: "From the Rubble",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Enchantment],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::IsSourceChosenCreatureType).from_your_graveyard()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Finality, amount: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}

/// Kinjalli's Sunwing — flying; creatures your opponents control enter
/// tapped.
pub fn kinjallis_sunwing() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Creatures your opponents control enter tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::EachPermanent(R::ControlledByOpponent.and(R::Creature)),
            },
        }],
        ..dino("Kinjalli's Sunwing", cost(&[generic(2), w()]), 2, 3)
    }
}

/// Majestic Heliopterus — flying; attacks: another target Dinosaur you
/// control gains flying until end of turn.
pub fn majestic_heliopterus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(yours(dinosaur()).and(R::OtherThanSource)),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..dino("Majestic Heliopterus", cost(&[generic(3), w()]), 2, 2)
    }
}

/// Marauding Raptor — creature spells you cast cost {1} less; another
/// creature of yours entering takes 2 from it, and a Dinosaur dealt damage
/// this way gives it +2/+0 until end of turn.
pub fn marauding_raptor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creature spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Creature, amount: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(2) },
                Effect::If {
                    cond: Predicate::SelectorCountAtLeast {
                        sel: Selector::DamagedThisResolution { filter: dinosaur() },
                        n: Value::ONE,
                    },
                    then: Box::new(eot_pump(Selector::This, Value::Const(2), Value::Const(0))),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..dino("Marauding Raptor", cost(&[generic(1), r()]), 2, 3)
    }
}

/// Progenitor's Icon — as it enters, choose a creature type; {T}: one mana
/// of any color; {T}: the next spell of that type you cast this turn has
/// flash.
pub fn progenitors_icon() -> CardDefinition {
    CardDefinition {
        name: "Progenitor's Icon",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        activated_abilities: vec![
            tap_add_any_color(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::NextSpellOfChosenTypeHasFlashThisTurn,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Raging Regisaur — attacks: 1 damage to any target.
pub fn raging_regisaur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack_ping_any(1)],
        ..dino("Raging Regisaur", cost(&[generic(2), r(), g()]), 4, 4)
    }
}

/// Raging Swordtooth — trample; ETB 1 damage to each other creature.
pub fn raging_swordtooth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
            amount: Value::ONE,
        })],
        ..dino("Raging Swordtooth", cost(&[generic(3), r(), g()]), 5, 5)
    }
}

/// Rampaging Brontodon — trample; attacks: +1/+1 until end of turn per land
/// you control.
pub fn rampaging_brontodon() -> CardDefinition {
    let lands = || Value::count(Selector::EachPermanent(yours(R::Land)));
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_attack(eot_pump(Selector::This, lands(), lands()))],
        ..dino("Rampaging Brontodon", cost(&[generic(5), g(), g()]), 7, 7)
    }
}

/// Runic Armasaur — whenever an opponent activates a non-mana ability of a
/// creature or land, you may draw a card.
pub fn runic_armasaur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivated, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.or(R::Land) },
            ),
            effect: Effect::MayDo {
                description: "Draw a card".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..dino("Runic Armasaur", cost(&[generic(1), g(), g()]), 2, 5)
    }
}

/// Savage Stomp — {2} less if it targets a Dinosaur you control; a +1/+1
/// counter on target creature you control, then it fights target creature
/// you don't control.
pub fn savage_stomp() -> CardDefinition {
    CardDefinition {
        name: "Savage Stomp",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        self_cost_reduction_if_target: Some((yours(R::Creature.and(dinosaur())), 2)),
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::TargetFiltered { slot: 0, filter: yours(R::Creature) },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::Fight {
                attacker: Selector::TargetFiltered { slot: 0, filter: yours(R::Creature) },
                defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
            },
        ]),
        ..Default::default()
    }
}

/// Scion of Calamity — myriad; combat damage to a player: destroy target
/// artifact or enchantment that player controls.
pub fn scion_of_calamity() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            myriad(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Destroy {
                    what: target_filtered(R::Artifact.or(R::Enchantment).and(R::ControlledByTriggerPlayer)),
                },
            },
        ],
        ..dino("Scion of Calamity", cost(&[generic(3), g(), g()]), 5, 5)
    }
}

/// Sunfrill Imitator — attacks: it may become a copy of another target
/// Dinosaur you control, keeping this ability. Residual: the copy takes the
/// copied Dinosaur's name.
pub fn sunfrill_imitator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Become a copy of another target Dinosaur you control".into(),
            body: Box::new(Effect::BecomeCopyOf {
                what: Selector::This,
                source: target_filtered(yours(dinosaur()).and(R::OtherThanSource)),
                extra_creature_types: vec![],
                keep_own_triggered: true,
                keep_own_activated: false,
            }),
        })],
        ..dino("Sunfrill Imitator", cost(&[generic(2), g()]), 3, 3)
    }
}

/// Temple Altisaur — damage to another Dinosaur you control is prevented
/// down to 1.
pub fn temple_altisaur() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If a source would deal damage to another Dinosaur you control, prevent all but 1 of that damage.",
            effect: StaticEffect::CapDamageToYourOtherMatchingCreatures { filter: dinosaur(), cap: 1 },
        }],
        ..dino("Temple Altisaur", cost(&[generic(4), w()]), 3, 4)
    }
}

/// Thunderherd Migration — reveal a Dinosaur card or pay {1}; a basic land
/// onto the battlefield tapped.
pub fn thunderherd_migration() -> CardDefinition {
    CardDefinition {
        name: "Thunderherd Migration",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::RevealFromHandOrPay { filter: dinosaur(), pay: 1 }],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        ..Default::default()
    }
}

/// Thundering Spineback — other Dinosaurs you control get +1/+1; {5}{G}: a
/// 3/3 green trampling Dinosaur.
pub fn thundering_spineback() -> CardDefinition {
    let token = Arc::new(TokenDefinition {
        name: "Dinosaur".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        keywords: vec![Keyword::Trample],
        ..Default::default()
    });
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Dinosaurs you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(yours(dinosaur()).and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g()]),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: token },
            ..Default::default()
        }],
        ..dino("Thundering Spineback", cost(&[generic(5), g(), g()]), 5, 5)
    }
}

/// Wakening Sun's Avatar — ETB, if cast from your hand: destroy all
/// non-Dinosaur creatures.
pub fn wakening_suns_avatar() -> CardDefinition {
    let mut wrath = etb(Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(dinosaur().negate())) });
    wrath.event = wrath.event.with_filter(Predicate::SourceCastFromOwnersHand);
    CardDefinition {
        triggered_abilities: vec![wrath],
        ..creature(
            "Wakening Sun's Avatar",
            cost(&[generic(5), w(), w(), w()]),
            vec![CreatureType::Dinosaur, CreatureType::Avatar],
            7,
            7,
        )
    }
}

/// Wayta, Trainer Prodigy — haste; {2}{G},{T}: target creature you control
/// fights another target creature, {2} less if both are yours; triggers a
/// creature of yours being dealt damage causes trigger an additional time.
pub fn wayta_trainer_prodigy() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "If a creature you control being dealt damage causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.",
            effect: StaticEffect::DoubleControllerCreatureDamagedTriggers,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), g()]),
            cost_reduction_if_targets: Some((yours(R::Creature), 2)),
            effect: Effect::Fight {
                attacker: Selector::TargetFiltered { slot: 0, filter: yours(R::Creature) },
                defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::OtherThanTargetSlot(0)) },
            },
            ..Default::default()
        }],
        ..creature(
            "Wayta, Trainer Prodigy",
            cost(&[r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            1,
            5,
        )
    }
}

/// Wrathful Raptors — trample; a Dinosaur you control dealt damage deals
/// that much to any target that isn't a Dinosaur. Residual: damage that
/// kills the Raptors in the same event doesn't trigger them.
pub fn wrathful_raptors() -> CardDefinition {
    let not_dino = || dinosaur().negate();
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: dinosaur() },
            ),
            effect: Effect::DealDamageFrom {
                source: Selector::TriggerSource,
                to: target_filtered(
                    R::Creature.and(not_dino()).or(R::Player).or(R::Planeswalker.and(not_dino())),
                ),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..dino("Wrathful Raptors", cost(&[generic(4), r()]), 5, 5)
    }
}

/// Zacama, Primal Calamity — reach, vigilance, trample; ETB, if cast:
/// untap all your lands; {2}{R}: 3 damage to target creature; {2}{G}:
/// destroy target artifact or enchantment; {2}{W}: gain 3 life.
pub fn zacama_primal_calamity() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Reach, Keyword::Vigilance, Keyword::Trample],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SourceWasCast,
            then: Box::new(Effect::Untap { what: Selector::EachPermanent(yours(R::Land)), up_to: None }),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(3) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g()]),
                effect: Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                ..Default::default()
            },
        ],
        ..creature(
            "Zacama, Primal Calamity",
            cost(&[generic(6), r(), g(), w()]),
            vec![CreatureType::Elder, CreatureType::Dinosaur],
            9,
            9,
        )
    }
}
