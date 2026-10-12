//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat, sixth file (`cmdr_edhrec5` is the
//! fifth). Hylda of the Icy Crown's tappers; Mr. House's dice. Tests in
//! `tests/recent_b/cmdr_edhrec6.rs`.

use crate::card::{
    Adventure, ArtifactSubtype, CardDefinition, CounterType, EquipBonus, TokenDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword,
    LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::card::Zone;
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered, tokens_gain_until_eot};
use crate::effect::{
    ActivatedAbility, Duration, Effect, KeywordAct, ManaPayload, PlayerRef, RevealMissDest, ZoneDest, ZoneRef,
};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, u, w, x, Color, ManaCost};
use crabomination_base::tokens::treasure_token;
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

// ── Mr. House, President and CEO (RWB) ───────────────────────────────────────

fn token(name: &str, p: i32, t: i32, color: Color, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(n: i32, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(def) }
}

/// Roll `count` `sides`-sided dice against `results` (CR 706.3).
fn roll(sides: u8, count: Value, results: Vec<(u8, u8, Effect)>) -> Effect {
    Effect::RollDie {
        sides,
        count,
        modifier: Value::Const(0),
        reroll_at_most: 0,
        ignore_lowest: 0,
        results,
        on_doubles: None,
    }
}

fn d20(results: Vec<(u8, u8, Effect)>) -> Effect {
    roll(20, Value::ONE, results)
}

/// "Whenever you roll one or more dice" (CR 706).
fn on_roll(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::RolledDice, EventScope::YourControl), effect }
}

fn until_eot(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn pump(what: Selector, p: Value, t: Value) -> Effect {
    Effect::PumpPT { what, power: p, toughness: t, duration: Duration::EndOfTurn }
}

/// Strength-Testing Hammer — {1} Equipment, equip {3}. Equipped creature
/// attacking rolls a d6 for +X/+0; then, holding the greatest power, it draws.
pub fn strength_testing_hammer() -> CardDefinition {
    CardDefinition {
        name: "Strength-Testing Hammer",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![on_attack(roll(
                6,
                Value::ONE,
                vec![(
                    1,
                    u8::MAX,
                    Effect::Seq(vec![
                        pump(Selector::This, Value::LastDieRoll, Value::ZERO),
                        Effect::If {
                            cond: Predicate::EntityMatches {
                                what: Selector::This,
                                filter: R::HasGreatestPowerAmongAllCreatures,
                            },
                            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                            else_: Box::new(Effect::Noop),
                        },
                    ]),
                )],
            ))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// The Deck of Many Things — {5} legendary Artifact. {2}, {T}: roll a d20 minus
/// your hand size; 0 or less discards your hand, 1-9 returns a random
/// graveyard card, 10-19 draws two, 20 reanimates a creature from any
/// graveyard whose death costs its owner the game.
pub fn the_deck_of_many_things() -> CardDefinition {
    let reanimate = Effect::Seq(vec![
        Effect::MoveChosen {
            from: Selector::EachMatching { zone: ZoneRef::Graveyard(PlayerRef::EachPlayer), filter: R::Creature },
            filter: None,
            count: Value::ONE,
            up_to: false,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        Effect::GrantTriggeredAbility {
            what: Selector::LastMoved,
            trigger: Box::new(on_dies(Effect::LoseGame { who: PlayerRef::OwnerOf(Box::new(Selector::This)) })),
            duration: Duration::Permanent,
        },
    ]);
    CardDefinition {
        name: "The Deck of Many Things",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            // CR 706.2 — the hand size is a modifier: a result of 0 or less
            // matches no arm and discards the hand instead.
            effect: Effect::RollDie {
                sides: 20,
                count: Value::ONE,
                modifier: Value::Diff(Box::new(Value::ZERO), Box::new(Value::HandSizeOf(PlayerRef::You))),
                reroll_at_most: 0,
                ignore_lowest: 0,
                results: vec![
                    (
                        0,
                        0,
                        Effect::Discard {
                            who: Selector::You,
                            amount: Value::HandSizeOf(PlayerRef::You),
                            random: false,
                        },
                    ),
                    (1, 9, Effect::ReturnRandomFromGraveyard { who: PlayerRef::You, filter: R::Any, count: Value::ONE }),
                    (10, 19, Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
                    (20, 20, reanimate),
                ],
                on_doubles: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Treasure Chest — {3} Artifact. {4}, sacrifice: roll a d20 — lose 3 life;
/// five Treasures; gain 3 and draw three; or tutor (an artifact may enter).
pub fn treasure_chest() -> CardDefinition {
    CardDefinition {
        name: "Treasure Chest",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            mana_cost: cost(&[generic(4)]),
            effect: d20(vec![
                (1, 1, Effect::LoseLife { who: Selector::You, amount: Value::Const(3) }),
                (2, 9, Effect::CreateToken { who: PlayerRef::You, count: Value::Const(5), definition: Arc::new(treasure_token()) }),
                (
                    10,
                    19,
                    Effect::Seq(vec![
                        Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                        Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                    ]),
                ),
                (
                    20,
                    u8::MAX,
                    Effect::Search {
                        who: PlayerRef::You,
                        filter: R::Any,
                        to: ZoneDest::IfCard {
                            filter: Box::new(R::Artifact),
                            then: Box::new(ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false }),
                            else_: Box::new(ZoneDest::Hand(PlayerRef::You)),
                        },
                    },
                ),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vexing Puzzlebox — {3} Artifact. Your rolls charge it by the result; {T}:
/// any color, then a d20; {T}, remove 100 charge counters: an artifact from
/// your library onto the battlefield. Residual: a multi-die roll charges by
/// its highest result (INCOMPLETE_CARDS).
pub fn vexing_puzzlebox() -> CardDefinition {
    CardDefinition {
        name: "Vexing Puzzlebox",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![on_roll(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Charge,
            amount: Value::TriggerEventAmount,
        })],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                    d20(vec![]),
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Charge, 100)),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Artifact,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Wand of Wonder — {3}{R} Artifact. {4}, {T}: roll a d20; each opponent
/// exiles to an instant or sorcery and shuffles the rest back; cast up to
/// X (1 / 2 / 3) of the exiled spells free.
pub fn wand_of_wonder() -> CardDefinition {
    let spells = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    let wonder = |x: i32| {
        Effect::Seq(vec![
            Effect::EachPlayerDoes {
                who: PlayerRef::EachOpponent,
                body: Box::new(Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: spells(),
                    to: ZoneDest::Exile,
                    cap: Value::Const(1000),
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::ShuffleIntoLibrary,
                }),
            },
            Effect::CastAnyOrderWithoutPaying {
                what: Selector::ExiledThisResolution { filter: spells() },
                source_zone: Zone::Exile,
                filter: None,
                cap: Some(Value::Const(x)),
                total_mana_value: None,
            },
        ])
    };
    CardDefinition {
        name: "Wand of Wonder",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4)]),
            effect: d20(vec![(1, 9, wonder(1)), (10, 19, wonder(2)), (20, u8::MAX, wonder(3))]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Brazen Dwarf — {1}{R} 1/3 Dwarf Shaman. Your rolls ping each opponent.
pub fn brazen_dwarf() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_roll(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
        })],
        ..creature("Brazen Dwarf", cost(&[generic(1), r()]), vec![CreatureType::Dwarf, CreatureType::Shaman], 1, 3)
    }
}

/// Celebr-8000 — {5} 3/3 Clown Robot artifact creature. Beginning of combat on
/// your turn, two d6: each 1 is +1/+1, each other face an ability (2 menace,
/// 3 vigilance, 4 lifelink, 5 flying, 6 indestructible); doubles add double
/// strike (CR 706.5).
pub fn celebr_8000() -> CardDefinition {
    let me = || Selector::This;
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::RollDie {
                sides: 6,
                count: Value::Const(2),
                modifier: Value::Const(0),
                reroll_at_most: 0,
                ignore_lowest: 0,
                results: vec![
                    (1, 1, pump(me(), Value::ONE, Value::ONE)),
                    (2, 2, until_eot(me(), Keyword::Menace)),
                    (3, 3, until_eot(me(), Keyword::Vigilance)),
                    (4, 4, until_eot(me(), Keyword::Lifelink)),
                    (5, 5, until_eot(me(), Keyword::Flying)),
                    (6, u8::MAX, until_eot(me(), Keyword::Indestructible)),
                ],
                on_doubles: Some(Box::new(until_eot(me(), Keyword::DoubleStrike))),
            },
        }],
        ..creature("Celebr-8000", cost(&[generic(5)]), vec![CreatureType::Clown, CreatureType::Robot], 3, 3)
    }
}

/// Hoarding Ogre — {3}{R} 3/3 Ogre. Attacking, a d20: one, two or three
/// Treasures.
pub fn hoarding_ogre() -> CardDefinition {
    let treasures = |n: i32| Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(treasure_token()) };
    CardDefinition {
        triggered_abilities: vec![on_attack(d20(vec![(1, 9, treasures(1)), (10, 19, treasures(2)), (20, u8::MAX, treasures(3))]))],
        ..creature("Hoarding Ogre", cost(&[generic(3), r()]), vec![CreatureType::Ogre], 3, 3)
    }
}

/// Iron Mastiff — {4} 4/4 Dog artifact creature. Attacking, a d20 per player
/// being attacked, keeping only the highest: it hits you, the defending
/// player, or each opponent for its power.
pub fn iron_mastiff() -> CardDefinition {
    let bite = |to: Selector| Effect::DealDamage { to, amount: Value::PowerOf(Box::new(Selector::This)) };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![on_attack(Effect::IgnoreAllButHighestRoll {
            dice: Value::OpponentsAttackedThisCombat,
            roll: Box::new(d20(vec![
                (1, 9, bite(Selector::You)),
                (10, 19, bite(Selector::Player(PlayerRef::DefendingPlayer))),
                (20, u8::MAX, bite(Selector::Player(PlayerRef::EachOpponent))),
            ])),
        })],
        ..creature("Iron Mastiff", cost(&[generic(4)]), vec![CreatureType::Dog], 4, 4)
    }
}

/// Lightfoot Rogue — {1}{B} 2/1 Halfling Rogue. Attacking, a d20: deathtouch;
/// +1/+0 too; or +3/+0, first strike too.
pub fn lightfoot_rogue() -> CardDefinition {
    let me = || Selector::This;
    CardDefinition {
        triggered_abilities: vec![on_attack(d20(vec![
            (1, 9, until_eot(me(), Keyword::Deathtouch)),
            (10, 19, Effect::Seq(vec![pump(me(), Value::ONE, Value::ZERO), until_eot(me(), Keyword::Deathtouch)])),
            (
                20,
                u8::MAX,
                Effect::Seq(vec![
                    pump(me(), Value::Const(3), Value::ZERO),
                    until_eot(me(), Keyword::FirstStrike),
                    until_eot(me(), Keyword::Deathtouch),
                ]),
            ),
        ]))],
        ..creature("Lightfoot Rogue", cost(&[generic(1), b()]), vec![CreatureType::Halfling, CreatureType::Rogue], 2, 1)
    }
}

/// Wyll, Blade of Frontiers — {1}{R} 1/1 legendary Human Warlock. Your rolls
/// add a die and ignore the lowest (CR 706.6); each roll grows Wyll. Choose a
/// Background.
pub fn wyll_blade_of_frontiers() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::ChooseABackground],
        static_abilities: vec![StaticAbility {
            description: "If you would roll one or more dice, instead roll that many dice plus one and ignore the lowest roll.",
            effect: StaticEffect::RollExtraDiceIgnoreLowest(1),
        }],
        triggered_abilities: vec![on_roll(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature(
            "Wyll, Blade of Frontiers",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            1,
            1,
        )
    }
}

/// Night Shift of the Living Dead — {3}{B} Enchantment. Once each turn you may
/// pay 1 life to move a die result by one (CR 706.2); every 6 you roll makes a
/// 2/2 Zombie Employee. Residual: a UI seat takes the headless shift
/// (INCOMPLETE_CARDS).
pub fn night_shift_of_the_living_dead() -> CardDefinition {
    let six = Predicate::All(vec![
        Predicate::ValueAtLeast(Value::TriggerEventAmount, Value::Const(6)),
        Predicate::ValueAtMost(Value::TriggerEventAmount, Value::Const(6)),
    ]);
    CardDefinition {
        name: "Night Shift of the Living Dead",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "After you roll a die, you may pay 1 life. If you do, increase or decrease the result by 1. Do this only once each turn.",
            effect: StaticEffect::PayLifeToShiftDieOnceEachTurn,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Performed(KeywordAct::DieResult), EventScope::YourControl).with_filter(six),
            effect: make(1, token("Zombie Employee", 2, 2, Color::Black, vec![CreatureType::Zombie, CreatureType::Employee])),
        }],
        ..Default::default()
    }
}

/// Contraband Livestock — {1}{W} Instant. Exile target creature; a d20 gives
/// its controller a 4/4 Ox, a 2/2 Boar or a 0/1 Goat.
pub fn contraband_livestock() -> CardDefinition {
    let theirs = |def: TokenDefinition| Effect::CreateToken {
        who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
        count: Value::ONE,
        definition: Arc::new(def),
    };
    spell(
        "Contraband Livestock",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Creature) },
            d20(vec![
                (1, 9, theirs(token("Ox", 4, 4, Color::Green, vec![CreatureType::Ox]))),
                (10, 19, theirs(token("Boar", 2, 2, Color::Green, vec![CreatureType::Boar]))),
                (20, u8::MAX, theirs(token("Goat", 0, 1, Color::White, vec![CreatureType::Goat]))),
            ]),
        ]),
    )
}

/// Six-Sided Die — {2}{B} Instant. Choose target creature and roll a d6: base
/// toughness 1; two -1/-1 counters; 3 damage and you gain 3; -4/-4; destroy;
/// or exile.
pub fn six_sided_die() -> CardDefinition {
    let it = || target_filtered(R::Creature);
    spell(
        "Six-Sided Die",
        cost(&[generic(2), b()]),
        CardType::Instant,
        roll(
                6,
                Value::ONE,
                vec![
                    (
                        1,
                        1,
                        Effect::SetBasePT {
                            what: it(),
                            power: Value::BasePowerOf(Box::new(it())),
                            toughness: Value::ONE,
                            duration: Duration::EndOfTurn,
                        },
                    ),
                    (2, 2, Effect::AddCounter { what: it(), kind: CounterType::MinusOneMinusOne, amount: Value::Const(2) }),
                    (
                        3,
                        3,
                        Effect::Seq(vec![
                            Effect::DealDamage { to: it(), amount: Value::Const(3) },
                            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                        ]),
                    ),
                    (4, 4, pump(it(), Value::Const(-4), Value::Const(-4))),
                    (5, 5, Effect::Destroy { what: it() }),
                    (6, u8::MAX, Effect::Exile { what: it() }),
                ],
            ),
    )
}

/// Wyll's Reversal — {2}{R} Instant. Target spell or ability with targets: a
/// d20 plus your greatest power; you may retarget it, and on 15+ copy it too.
pub fn wylls_reversal() -> CardDefinition {
    let retarget = || Effect::ChooseNewTargetsForSpell {
        what: target_filtered(R::IsSpellOnStack.or(R::HasAbilityOnStack).and(R::SpellWithAnyTarget)),
    };
    let copy = Effect::If {
        cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::IsSpellOnStack },
        then: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::Target(0), count: Value::ONE }),
        else_: Box::new(Effect::CopyAbility { what: Selector::Target(0), times: Value::ONE }),
    };
    spell(
        "Wyll's Reversal",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::RollDie {
                sides: 20,
                count: Value::ONE,
                modifier: Value::GreatestPowerControlled { who: PlayerRef::You },
                reroll_at_most: 0,
                ignore_lowest: 0,
                results: vec![(1, 14, retarget()), (15, u8::MAX, Effect::Seq(vec![retarget(), copy]))],
                on_doubles: None,
            },
    )
}

/// Comet, Stellar Pup — {2}{R}{W} loyalty 5. 0: roll a d6 — +2 and two hasty
/// Squirrels; −1 and a card with mana value 2 or less back; his loyalty in
/// damage to a creature or player, then −2; or +1 and two more activations.
/// Nothing targets (2022-10-07 ruling).
pub fn comet_stellar_pup() -> CardDefinition {
    let me = || Selector::This;
    let loyalty = |n: i32| {
        if n >= 0 {
            Effect::AddCounter { what: me(), kind: CounterType::Loyalty, amount: Value::Const(n) }
        } else {
            Effect::RemoveCounter { what: me(), kind: CounterType::Loyalty, amount: Value::Const(-n) }
        }
    };
    let squirrels = tokens_gain_until_eot(
        make(2, token("Squirrel", 1, 1, Color::Green, vec![CreatureType::Squirrel])),
        &[Keyword::Haste],
    );
    CardDefinition {
        name: "Comet, Stellar Pup",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Comet], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: 0,
            effect: roll(
                6,
                Value::ONE,
                vec![
                    (1, 2, Effect::Seq(vec![loyalty(2), squirrels])),
                    (
                        3,
                        3,
                        Effect::Seq(vec![
                            loyalty(-1),
                            Effect::MoveChosen {
                                from: Selector::EachMatching {
                                    zone: ZoneRef::Graveyard(PlayerRef::You),
                                    filter: R::ManaValueAtMost(2),
                                },
                                filter: None,
                                count: Value::ONE,
                                up_to: false,
                                to: ZoneDest::Hand(PlayerRef::You),
                            },
                        ]),
                    ),
                    (
                        4,
                        5,
                        Effect::Seq(vec![
                            Effect::DealDamageToChosenCreatureOrPlayer {
                                amount: Value::CountersOn { what: Box::new(me()), kind: CounterType::Loyalty },
                            },
                            loyalty(-2),
                        ]),
                    ),
                    (6, u8::MAX, Effect::Seq(vec![loyalty(1), Effect::LoyaltyTwoMoreTimesThisTurn { what: me() }])),
                ],
            ),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Attempted Murder — {X}{B}{B} Sorcery. Choose target creature, roll X d6:
/// each even result two -1/-1 counters on it, each odd one a 1/2 flying Storm
/// Crow.
pub fn attempted_murder() -> CardDefinition {
    let wither = || Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::MinusOneMinusOne, amount: Value::Const(2) };
    let crow = || {
        make(1, TokenDefinition {
            keywords: vec![Keyword::Flying],
            ..token("Storm Crow", 1, 2, Color::Blue, vec![CreatureType::Bird])
        })
    };
    spell(
        "Attempted Murder",
        cost(&[x(), b(), b()]),
        CardType::Sorcery,
        roll(
                6,
                Value::XFromCost,
                vec![(1, 1, crow()), (2, 2, wither()), (3, 3, crow()), (4, 4, wither()), (5, 5, crow()), (6, 6, wither())],
            ),
    )
}

/// Circuits Act — {2}{R} Sorcery. Three d6: a 1/1 Clown Robot per different
/// result.
pub fn circuits_act() -> CardDefinition {
    let clown = TokenDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..token("Clown Robot", 1, 1, Color::White, vec![CreatureType::Clown, CreatureType::Robot])
    };
    spell(
        "Circuits Act",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            roll(6, Value::Const(3), vec![]),
            Effect::CreateToken { who: PlayerRef::You, count: Value::LastRollDistinctResults, definition: Arc::new(clown) },
        ]),
    )
}

/// Myrkul's Edict — {1}{B} Sorcery. A d20: a chosen opponent sacrifices a
/// creature; each opponent does; or each sacrifices their greatest power.
pub fn myrkuls_edict() -> CardDefinition {
    let sac = |who: Selector, filter: R| Effect::Sacrifice { who, count: Value::ONE, filter };
    spell(
        "Myrkul's Edict",
        cost(&[generic(1), b()]),
        CardType::Sorcery,
        d20(vec![
            (
                1,
                9,
                Effect::ChooseOpponentThen {
                    then: Box::new(sac(Selector::Player(PlayerRef::ChosenPlayerOfSource), R::Creature)),
                },
            ),
            (10, 19, sac(Selector::Player(PlayerRef::EachOpponent), R::Creature)),
            (
                20,
                u8::MAX,
                sac(
                    Selector::Player(PlayerRef::EachOpponent),
                    R::Creature.and(R::HasGreatestPowerAmongControlled(Box::new(R::Creature))),
                ),
            ),
        ]),
    )
}

/// Recruitment Drive — {2}{W} Sorcery. A d20: two 1/1 Soldiers, two 2/2
/// Knights, or three Knights.
pub fn recruitment_drive() -> CardDefinition {
    let soldier = || token("Soldier", 1, 1, Color::White, vec![CreatureType::Soldier]);
    let knight = || token("Knight", 2, 2, Color::White, vec![CreatureType::Knight]);
    spell(
        "Recruitment Drive",
        cost(&[generic(2), w()]),
        CardType::Sorcery,
        d20(vec![(1, 9, make(2, soldier())), (10, 19, make(2, knight())), (20, u8::MAX, make(3, knight()))]),
    )
}

/// Thunderwave — {2}{R}{R} Sorcery. A d20: 3 to each creature; 3 to each but
/// one you may choose; or 6 to each creature your opponents control.
pub fn thunderwave() -> CardDefinition {
    let every = || Selector::EachMatching { zone: ZoneRef::Battlefield, filter: R::Creature };
    spell(
        "Thunderwave",
        cost(&[generic(2), r(), r()]),
        CardType::Sorcery,
        d20(vec![
            (1, 9, Effect::DealDamage { to: every(), amount: Value::Const(3) }),
            (
                10,
                19,
                Effect::ChooseSomeAmong {
                    what: every(),
                    chooser: PlayerRef::You,
                    count: Value::ONE,
                    up_to: true,
                    chosen: Box::new(Effect::Noop),
                    other: Box::new(Effect::DealDamage { to: Selector::SeparatedPile { chosen: false }, amount: Value::Const(3) }),
                },
            ),
            (20, u8::MAX, Effect::DealDamage { to: their_creatures(), amount: Value::Const(6) }),
        ]),
    )
}
