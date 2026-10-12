//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat, sixth file (`cmdr_edhrec5` is the
//! fifth). Hylda of the Icy Crown's tappers. Tests in
//! `tests/recent_b/cmdr_edhrec6.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword,
    LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, PlayerRef, ZoneDest, ZoneRef};
use crate::game::types::TurnStep;
use crate::mana::{cost, generic, u, w, x, ManaCost};

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn tap(what: Selector) -> Effect {
    Effect::Tap { what }
}

/// Tap target `slot` (a creature), as one arm of an "up to N" list.
fn tap_slot(slot: u8) -> Effect {
    tap(Selector::TargetFiltered { slot, filter: R::Creature })
}

/// "Tap up to `n` target creatures."
fn tap_up_to(n: u8) -> Effect {
    Effect::OptionalTargets { min: 0, body: Box::new(Effect::Seq((0..n).map(tap_slot).collect())) }
}

/// Tap `Target(0)`; it doesn't untap during its controller's next untap step.
fn tap_and_hold() -> Effect {
    Effect::Seq(vec![tap(Selector::Target(0)), Effect::SkipNextUntap { what: Selector::Target(0) }])
}

/// Every creature your opponents control.
fn their_creatures() -> Selector {
    Selector::EachMatching { zone: ZoneRef::Battlefield, filter: R::Creature.and(R::ControlledByOpponent) }
}

// ── Hylda of the Icy Crown (WU) ──────────────────────────────────────────────

/// Hylda's Crown of Winter — {3} legendary Artifact. {1}, {T}: tap target
/// creature, {1} cheaper on your turn; {3}, sacrifice: draw per tapped creature
/// your opponents control.
pub fn hyldas_crown_of_winter() -> CardDefinition {
    CardDefinition {
        name: "Hylda's Crown of Winter",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                cost_reduction_if: Some((Predicate::IsTurnOf(PlayerRef::You), 1)),
                effect: tap(target_filtered(R::Creature)),
                ..Default::default()
            },
            ActivatedAbility {
                sac_cost: true,
                mana_cost: cost(&[generic(3)]),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::count(Selector::EachMatching {
                        zone: ZoneRef::Battlefield,
                        filter: R::Creature.and(R::Tapped).and(R::ControlledByOpponent),
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Dreamshackle Geist — {1}{U}{U} 3/1 flying Spirit. At the beginning of combat
/// on your turn, choose up to one: tap target creature, or target creature
/// doesn't untap during its controller's next untap step.
pub fn dreamshackle_geist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            // "Choose up to one —": a declined target is the "none".
            effect: Effect::ChooseMode(vec![
                Effect::OptionalTargets { min: 0, body: Box::new(tap(target_filtered(R::Creature))) },
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::SkipNextUntap { what: target_filtered(R::Creature) }),
                },
            ]),
        }],
        ..creature("Dreamshackle Geist", cost(&[generic(1), u(), u()]), vec![CreatureType::Spirit], 3, 1)
    }
}

/// Elvish Mariner — {2}{U} 3/2 Elf Pilot. Attacking, scry 1; whenever you scry,
/// tap up to X target nonland permanents, X the cards looked at.
pub fn elvish_mariner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::Scry { who: PlayerRef::You, amount: Value::ONE }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Scried, EventScope::YourControl),
                // X is the scry's `looked_at` (a short library looks at fewer).
                effect: Effect::CapTargetsAt {
                    amount: Value::TriggerEventAmount,
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 8,
                        min_targets: 0,
                        filter: R::Permanent.and(R::Nonland),
                        effect: Box::new(tap(Selector::Target(0))),
                    }),
                },
            },
        ],
        ..creature("Elvish Mariner", cost(&[generic(2), u()]), vec![CreatureType::Elf, CreatureType::Pilot], 3, 2)
    }
}

/// Giant Killer — {W} 1/2 Human Peasant. {1}{W}, {T}: tap target creature.
/// Adventure — Chop Down {2}{W}: destroy target creature with power 4 or more.
pub fn giant_killer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), w()]),
            effect: tap(target_filtered(R::Creature)),
            ..Default::default()
        }],
        adventure: Some(Box::new(Adventure {
            name: "Chop Down",
            cost: cost(&[generic(2), w()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::PowerAtLeast(4))) },
        })),
        ..creature("Giant Killer", cost(&[w()]), vec![CreatureType::Human, CreatureType::Peasant], 1, 2)
    }
}

/// Githzerai Monk — {4}{W} 3/2 flash flying Gith Monk; entering, it taps every
/// creature you don't control.
pub fn githzerai_monk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(tap(Selector::EachMatching {
            zone: ZoneRef::Battlefield,
            filter: R::Creature.and(R::Not(Box::new(R::ControlledByYou))),
        }))],
        ..creature("Githzerai Monk", cost(&[generic(4), w()]), vec![CreatureType::Gith, CreatureType::Monk], 3, 2)
    }
}

/// Hammers of Moradin — {2}{W} 3/3 Dwarf Cleric with myriad; attacking, for
/// each opponent it taps up to one target creature that player controls.
pub fn hammers_of_moradin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: Effect::Myriad },
            // CR 601.2c — one target per opponent, none sharing a controller.
            on_attack(Effect::ForEachOpponentTarget {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 15,
                    min_targets: 0,
                    filter: R::Creature.and(R::ControlledByOpponent),
                    effect: Box::new(tap(Selector::Target(0))),
                }),
            }),
        ],
        ..creature("Hammers of Moradin", cost(&[generic(2), w()]), vec![CreatureType::Dwarf, CreatureType::Cleric], 3, 3)
    }
}

/// Illithid Harvester — {4}{U} 4/4 Horror. Entering, turn any number of target
/// tapped nontoken creatures face down as 2/2 Horrors. Adventure — Plant
/// Tadpoles {X}{U}{U}: tap X target creatures; they skip their next untap.
pub fn illithid_harvester() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 20,
            min_targets: 0,
            filter: R::Creature.and(R::Tapped).and(R::Not(Box::new(R::IsToken))),
            effect: Box::new(Effect::TurnFaceDownAs {
                what: Selector::Target(0),
                creature_type: CreatureType::Horror,
            }),
        })],
        adventure: Some(Box::new(Adventure {
            name: "Plant Tadpoles",
            cost: cost(&[x(), u(), u()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 20,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(tap_and_hold()),
                }),
            },
        })),
        ..creature("Illithid Harvester", cost(&[generic(4), u()]), vec![CreatureType::Horror], 4, 4)
    }
}

/// Topplegeist — {W} 1/1 flying Spirit. Entering, tap target creature an
/// opponent controls; delirium — each opponent's upkeep, tap one of theirs.
pub fn topplegeist() -> CardDefinition {
    let delirium = || Predicate::DeliriumActive { who: PlayerRef::You };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(tap(target_filtered(R::Creature.and(R::ControlledByOpponent)))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::OpponentControl)
                    .with_filter(delirium()),
                // CR 603.4 — the intervening "if" is checked again on resolution.
                effect: Effect::If {
                    cond: delirium(),
                    then: Box::new(tap(target_filtered(R::Creature.and(R::ControlledByActivePlayer)))),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..creature("Topplegeist", cost(&[w()]), vec![CreatureType::Spirit], 1, 1)
    }
}

/// Willbreaker — {3}{U}{U} 2/3 Human Wizard. Whenever a creature an opponent
/// controls becomes the target of a spell or ability you control, gain control
/// of it for as long as you control Willbreaker.
pub fn willbreaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::OpponentCreatureTargetedByYou),
            effect: Effect::GainControlWhileYouControlSource { what: Selector::TriggerSource },
        }],
        ..creature("Willbreaker", cost(&[generic(3), u(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 3)
    }
}

/// Time of Ice — {3}{U} Saga. I, II: tap target creature an opponent controls;
/// it doesn't untap while you control this Saga. III: return all tapped
/// creatures to their owners' hands.
pub fn time_of_ice() -> CardDefinition {
    let lock = || Effect::TapAndLockWhileSourcePresent {
        what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        while_you_control: true,
    };
    CardDefinition {
        name: "Time of Ice",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, lock()),
            (2, lock()),
            (
                3,
                Effect::Move {
                    what: Selector::EachMatching { zone: ZoneRef::Battlefield, filter: R::Creature.and(R::Tapped) },
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Downpour — {1}{U} Instant. Tap up to three target creatures.
pub fn downpour() -> CardDefinition {
    spell("Downpour", cost(&[generic(1), u()]), CardType::Instant, tap_up_to(3))
}

/// Feeling of Dread — {1}{W} Instant. Tap up to two target creatures.
/// Flashback {1}{U}.
pub fn feeling_of_dread() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), u()]))],
        ..spell("Feeling of Dread", cost(&[generic(1), w()]), CardType::Instant, tap_up_to(2))
    }
}

/// Icy Blast — {X}{U} Instant. Tap X target creatures; ferocious — they don't
/// untap during their controllers' next untap steps.
pub fn icy_blast() -> CardDefinition {
    spell(
        "Icy Blast",
        cost(&[x(), u()]),
        CardType::Instant,
        Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 20,
                min_targets: 0,
                filter: R::Creature,
                // Ferocious is checked as the spell resolves (2014-09-20 ruling).
                effect: Box::new(Effect::Seq(vec![
                    tap(Selector::Target(0)),
                    Effect::If {
                        cond: Predicate::FerociousActive { who: PlayerRef::You },
                        then: Box::new(Effect::SkipNextUntap { what: Selector::Target(0) }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            }),
        },
    )
}

/// Urza's Rebuff — {1}{U}{U} Instant. Choose one: counter target spell; or tap
/// up to two target creatures.
pub fn urzas_rebuff() -> CardDefinition {
    spell(
        "Urza's Rebuff",
        cost(&[generic(1), u(), u()]),
        CardType::Instant,
        Effect::ChooseMode(vec![Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) }, tap_up_to(2)]),
    )
}

/// You See a Guard Approach — {U} Instant. Choose one: tap target creature; or
/// target creature you control gains hexproof until end of turn.
pub fn you_see_a_guard_approach() -> CardDefinition {
    spell(
        "You See a Guard Approach",
        cost(&[u()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            tap(target_filtered(R::Creature)),
            Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Tamiyo, the Moon Sage — {3}{U}{U} loyalty 4. +1: tap target permanent; it
/// skips its next untap. −2: draw per tapped creature target player controls.
/// −8: an emblem — no maximum hand size, and cards put into your graveyard
/// from anywhere may return to your hand.
pub fn tamiyo_the_moon_sage() -> CardDefinition {
    CardDefinition {
        name: "Tamiyo, the Moon Sage",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Tamiyo], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    tap(target_filtered(R::Permanent)),
                    Effect::SkipNextUntap { what: Selector::Target(0) },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::TargetPlayerThen {
                    filter: R::Player,
                    then: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: Value::PermanentCountControlledByMatching(
                            PlayerRef::Target(0),
                            R::Creature.and(R::Tapped),
                        ),
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Tamiyo, the Moon Sage".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::YourControl),
                        effect: Effect::MayDo {
                            description: "Tamiyo's emblem: return it to your hand?".into(),
                            body: Box::new(Effect::Move {
                                what: Selector::TriggerSource,
                                to: ZoneDest::Hand(PlayerRef::You),
                            }),
                        },
                    }],
                    statics: vec![StaticAbility {
                        description: "You have no maximum hand size.",
                        effect: StaticEffect::NoMaximumHandSize,
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Cone of Cold — {3}{U} Sorcery. Roll a d20 and tap all creatures your
/// opponents control; 10+ they skip their next untap; 20 and until your next
/// turn, creatures your opponents control enter tapped.
pub fn cone_of_cold() -> CardDefinition {
    let tap_all = || tap(their_creatures());
    let hold = || Effect::SkipNextUntap { what: their_creatures() };
    spell(
        "Cone of Cold",
        cost(&[generic(3), u()]),
        CardType::Sorcery,
        Effect::RollDie {
            sides: 20,
            count: Value::ONE,
            modifier: Value::Const(0),
            reroll_at_most: 0,
            ignore_lowest: 0,
            results: vec![
                (1, 9, tap_all()),
                (10, 19, Effect::Seq(vec![tap_all(), hold()])),
                (
                    20,
                    20,
                    Effect::Seq(vec![tap_all(), hold(), Effect::OpponentsCreaturesEnterTappedUntilYourNextTurn]),
                ),
            ],
            on_doubles: None,
        },
    )
}

/// Theft of Dreams — {2}{U} Sorcery. Draw a card for each tapped creature
/// target opponent controls.
pub fn theft_of_dreams() -> CardDefinition {
    spell(
        "Theft of Dreams",
        cost(&[generic(2), u()]),
        CardType::Sorcery,
        Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::PermanentCountControlledByMatching(PlayerRef::Target(0), R::Creature.and(R::Tapped)),
            }),
        },
    )
}
