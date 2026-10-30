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
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, x, Color, ManaCost};
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

/// Vexing Puzzlebox — {3} Artifact. Your rolls charge it by the result (a
/// multi-die roll's result is the total of the dice kept, modifiers added —
/// counted die by die off `DieResult`; the planar die has none); {T}: any
/// color, then a d20; {T}, remove 100 charge counters: an artifact from your
/// library onto the battlefield.
pub fn vexing_puzzlebox() -> CardDefinition {
    CardDefinition {
        name: "Vexing Puzzlebox",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Performed(crate::effect::KeywordAct::DieResult), EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::TriggerEventAmount },
        }],
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
                exile_after: false,
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

// ── Ezio Auditore da Firenze (WUBRG) ─────────────────────────────────────────

fn assassin() -> R {
    R::HasCreatureType(CreatureType::Assassin)
}

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn human_assassin(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    legend(name, mana, vec![CreatureType::Human, CreatureType::Assassin], p, t)
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

/// A 1/1 black Assassin creature token with menace.
fn assassin_token() -> TokenDefinition {
    TokenDefinition { keywords: vec![Keyword::Menace], ..token("Assassin", 1, 1, Color::Black, vec![CreatureType::Assassin]) }
}

/// "Whenever [this] deals combat damage to a player".
fn on_hit(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

/// "Whenever one or more [filter] you control deal combat damage to a player".
fn yours_hit(filter: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
            .once_per_batch()
            .dealt_by(filter),
        effect,
    }
}

fn unblockable_this_turn(what: Selector) -> Effect {
    until_eot(what, Keyword::Unblockable)
}

/// Ezio Auditore da Firenze — {1}{B} 3/2 legendary Human Assassin, menace.
/// Assassin spells you cast have freerunning {B}{B} (CR 702.173); connecting
/// with a player at 10 or less life, pay {W}{U}{B}{R}{G} and they lose.
pub fn ezio_auditore_da_firenze() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![static_ab(
            "Assassin spells you cast have freerunning {B}{B}.",
            StaticEffect::GrantFreerunningToSpells { filter: assassin(), cost: cost(&[b(), b()]) },
        )],
        triggered_abilities: vec![on_hit(Effect::If {
            cond: Predicate::PlayerLifeAtMost { who: PlayerRef::TriggerEventPlayer, life: 10 },
            then: Box::new(Effect::MayPay {
                description: "pay {W}{U}{B}{R}{G}: that player loses the game".into(),
                mana_cost: cost(&[w(), u(), b(), r(), g()]),
                body: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::LoseGame { who: PlayerRef::TriggerEventPlayer }),
                }),
                else_: None,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..human_assassin("Ezio Auditore da Firenze", cost(&[generic(1), b()]), 3, 2)
    }
}

/// Haystack — {1}{W} Artifact. {2}, {T}: target creature you control phases out.
pub fn haystack() -> CardDefinition {
    CardDefinition {
        name: "Haystack",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PhaseOut {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                until_source_leaves: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hidden Blade — {2} Equipment with flash, equip {2}: +1/+0 and first strike;
/// entering, it attaches to target creature you control, an Assassin of
/// which gains deathtouch until end of turn.
pub fn hidden_blade() -> CardDefinition {
    let host = || target_filtered(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Hidden Blade",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 1, keywords: vec![Keyword::FirstStrike], ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AttachSourceTo { host: host() },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: assassin() },
                then: Box::new(until_eot(Selector::Target(0), Keyword::Deathtouch)),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..Default::default()
    }
}

/// Smoke Bomb — {3} Artifact with flash. All creatures have shroud; at your
/// upkeep you sacrifice it, and when you do a creature of yours can't be
/// blocked this turn.
pub fn smoke_bomb() -> CardDefinition {
    CardDefinition {
        name: "Smoke Bomb",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Flash],
        static_abilities: vec![static_ab(
            "All creatures have shroud.",
            StaticEffect::GrantKeyword { applies_to: Selector::EachPermanent(R::Creature), keyword: Keyword::Shroud },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::SacrificePermanent { what: Selector::This },
                Effect::Reflexive {
                    body: Box::new(unblockable_this_turn(target_filtered(R::Creature.and(R::ControlledByYou)))),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// The Animus — {2} legendary Artifact. Your end step exiles up to one
/// legendary creature card from a graveyard with a memory counter; {T}: until
/// your next turn, a legendary creature of yours becomes a copy of a creature
/// card in exile with a memory counter. Sorcery speed.
pub fn the_animus() -> CardDefinition {
    let legendary = || R::Creature.and(R::HasSupertype(Supertype::Legendary));
    CardDefinition {
        name: "The Animus",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: legendary().and(R::InGraveyard),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile },
                    Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Memory, amount: Value::ONE },
                ])),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::BecomeCopyOfFor {
                what: target_filtered(legendary().and(R::ControlledByYou)),
                source: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::InExile).and(R::WithCounter(CounterType::Memory)),
                },
                duration: Duration::UntilNextTurn,
                non_legendary: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Abstergo Entertainment — legendary land. {T}: {C}; {1}, {T}: any color;
/// {3}, {T}, exile it: return up to one historic card from your graveyard to
/// your hand, then exile all graveyards.
pub fn abstergo_entertainment() -> CardDefinition {
    CardDefinition {
        name: "Abstergo Entertainment",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                exile_self_cost: true,
                mana_cost: cost(&[generic(3)]),
                effect: Effect::Seq(vec![
                    Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 0,
                        filter: R::historic().from_your_graveyard(),
                        effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
                    },
                    Effect::ExileAllGraveyards { filter: None, opponents_only: false },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Brotherhood Headquarters — Land. {T}: {C}; {T}: any color, spent only on
/// Assassin spells, freerunning spells and Assassin sources' abilities.
pub fn brotherhood_headquarters() -> CardDefinition {
    CardDefinition {
        name: "Brotherhood Headquarters",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                        crate::mana::SpendRestriction::AssassinOrFreerunning,
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Altaïr Ibn-La'Ahad — {R}{W}{B} 3/3 first strike. Attacking, it exiles up
/// to one Assassin creature card from your graveyard with a memory counter,
/// then makes a tapped, attacking token copy of each memory creature card you
/// own in exile, exiled at end of combat.
pub fn altair_ibn_la_ahad() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(assassin()).and(R::InYourGraveyard),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile },
                    Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Memory, amount: Value::ONE },
                ])),
            },
            Effect::ForEach {
                selector: Selector::EachMatching {
                    zone: ZoneRef::Exile,
                    filter: R::Creature.and(R::OwnedByYou).and(R::WithCounter(CounterType::Memory)),
                },
                body: Box::new(Effect::TokenCopyAttackingUntilEndOfCombat { source: Selector::TriggerSource, sacrifice: false }),
            },
        ]))],
        ..human_assassin("Altaïr Ibn-La'Ahad", cost(&[r(), w(), b()]), 3, 3)
    }
}

/// Arno Dorian — {2}{B}{R} 3/3 deathtouch; other Assassins you control get
/// +2/+0. Disguise {B}{R}.
pub fn arno_dorian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Disguise(cost(&[b(), r()]))],
        static_abilities: vec![static_ab(
            "Other Assassins you control get +2/+0.",
            StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(assassin()).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 2,
                toughness: 0,
            },
        )],
        ..human_assassin("Arno Dorian", cost(&[generic(2), b(), r()]), 3, 3)
    }
}

/// Assassin Initiate — {B} 1/1 Human Assassin. {1}: flying, deathtouch or
/// lifelink until end of turn.
pub fn assassin_initiate() -> CardDefinition {
    let me = || Selector::This;
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::ChooseMode(vec![
                until_eot(me(), Keyword::Flying),
                until_eot(me(), Keyword::Deathtouch),
                until_eot(me(), Keyword::Lifelink),
            ]),
            ..Default::default()
        }],
        ..creature("Assassin Initiate", cost(&[b()]), vec![CreatureType::Human, CreatureType::Assassin], 1, 1)
    }
}

/// Aveline de Grandpré — {2}{G}{G} 3/3 deathtouch; a deathtouch creature of
/// yours connecting gets that many +1/+1 counters. Disguise {B}{G}.
pub fn aveline_de_grandpre() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Disguise(cost(&[b(), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .dealt_by(R::Creature.and(R::HasKeyword(Keyword::Deathtouch))),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..human_assassin("Aveline de Grandpré", cost(&[generic(2), g(), g()]), 3, 3)
    }
}

/// Aya of Alexandria — {2}{R}{W} 4/3 menace, lifelink; each historic creature
/// of yours connecting makes a 1/1 Assassin with menace.
pub fn aya_of_alexandria() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .dealt_by(R::Creature.and(R::historic())),
            effect: make(1, assassin_token()),
        }],
        ..human_assassin("Aya of Alexandria", cost(&[generic(2), r(), w()]), 4, 3)
    }
}

/// Bayek of Siwa — {3}{R}{W} 3/4 double strike; during your turn your other
/// historic creatures have double strike. Disguise {1}{R}{W}.
pub fn bayek_of_siwa() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike, Keyword::Disguise(cost(&[generic(1), r(), w()]))],
        static_abilities: vec![static_ab(
            "During your turn, other historic creatures you control have double strike.",
            StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::historic()).and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    keyword: Keyword::DoubleStrike,
                }),
            },
        )],
        ..human_assassin("Bayek of Siwa", cost(&[generic(3), r(), w()]), 3, 4)
    }
}

/// Desmond Miles — {1}{B} 1/3 menace; +1/+0 per other Assassin you control
/// and Assassin card in your graveyard; connecting, surveil the damage dealt.
pub fn desmond_miles() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![static_ab(
            "Desmond Miles gets +1/+0 for each other Assassin you control and each Assassin card in your graveyard.",
            StaticEffect::PumpPTByValue {
                applies_to: Selector::This,
                power: Value::Sum(vec![
                    Value::count(Selector::EachPermanent(
                        R::Creature.and(assassin()).and(R::ControlledByYou).and(R::OtherThanSource),
                    )),
                    Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: assassin() },
                ]),
                toughness: Value::ZERO,
            },
        )],
        triggered_abilities: vec![on_hit(Effect::Surveil { who: PlayerRef::You, amount: Value::TriggerEventAmount })],
        ..human_assassin("Desmond Miles", cost(&[generic(1), b()]), 1, 3)
    }
}

/// Etrata, the Silencer — {2}{U}{B} 3/5 legendary Vampire Assassin, can't be
/// blocked. Connecting, it exiles a creature that player controls with a hit
/// counter (three of their cards so marked and they lose), then shuffles
/// itself into its owner's library.
pub fn etrata_the_silencer() -> CardDefinition {
    let marked = Selector::CardsInZone {
        who: PlayerRef::TriggerEventPlayer,
        zone: Zone::Exile,
        filter: R::WithCounter(CounterType::Hit),
    };
    CardDefinition {
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![on_hit(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                to: ZoneDest::Exile,
            },
            Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Hit, amount: Value::ONE },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::count(marked), Value::Const(3)),
                then: Box::new(Effect::LoseGame { who: PlayerRef::TriggerEventPlayer }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Move {
                what: Selector::This,
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::This)),
                    pos: crate::effect::LibraryPosition::Shuffled,
                },
            },
        ]))],
        ..legend(
            "Etrata, the Silencer",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Vampire, CreatureType::Assassin],
            3,
            5,
        )
    }
}

/// Evie Frye — {1}{U} 2/1, partner with Jacob Frye. {1}, {T}: loot; a creature
/// card discarded this way makes a creature of yours unblockable this turn.
pub fn evie_frye() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Jacob Frye".into())],
        triggered_abilities: vec![crate::effect::shortcut::partner_with_search("Jacob Frye")],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::CreatureCardsDiscardedThisEffect, Value::ONE),
                    then: Box::new(Effect::Reflexive {
                        body: Box::new(unblockable_this_turn(target_filtered(R::Creature.and(R::ControlledByYou)))),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..human_assassin("Evie Frye", cost(&[generic(1), u()]), 2, 1)
    }
}

/// Ezio, Blade of Vengeance — {3}{U}{B} 5/5 deathtouch; each Assassin of
/// yours connecting draws a card.
pub fn ezio_blade_of_vengeance() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .dealt_by(R::Creature.and(assassin())),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..human_assassin("Ezio, Blade of Vengeance", cost(&[generic(3), u(), b()]), 5, 5)
    }
}

/// Jacob Frye — {2}{B} 3/2, partner with Evie Frye. Assassins of yours
/// connecting exile up to one Assassin or freerunning card from your
/// graveyard; you may cast a copy of it (paying its cost).
pub fn jacob_frye() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Evie Frye".into())],
        triggered_abilities: vec![
            crate::effect::shortcut::partner_with_search("Evie Frye"),
            yours_hit(
                R::Creature.and(assassin()),
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: assassin().or(R::HasFreerunning).and(R::InYourGraveyard),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile },
                        Effect::CastWithoutPayingImmediate {
                            what: Selector::LastMoved,
                            source_zone: Zone::Exile,
                            exile_after: false,
                            copy: true,
                            reduce_generic: 0,
                            pay_own_cost: true,
                        },
                    ])),
                },
            ),
        ],
        ..human_assassin("Jacob Frye", cost(&[generic(2), b()]), 3, 2)
    }
}

/// Layla Hassan — {3}{W} 3/4 first strike. Entering, and whenever Assassins
/// of yours connect, return target historic card from your graveyard to hand.
pub fn layla_hassan() -> CardDefinition {
    let regain = || Effect::Move {
        what: target_filtered(R::historic().from_your_graveyard()),
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![etb(regain()), yours_hit(R::Creature.and(assassin()), regain())],
        ..human_assassin("Layla Hassan", cost(&[generic(3), w()]), 3, 4)
    }
}

/// Lydia Frye — {2}{U/B} 3/2, can't be blocked by power 3 or greater; your end
/// step surveils one per tapped Assassin you control.
pub fn lydia_frye() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByPowerAtLeast(3)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::count(Selector::EachPermanent(
                    R::Creature.and(assassin()).and(R::Tapped).and(R::ControlledByYou),
                )),
            },
        }],
        ..human_assassin("Lydia Frye", cost(&[generic(2), hybrid(Color::Blue, Color::Black)]), 3, 2)
    }
}

/// Ramses, Assassin Lord — {2}{U}{B} 4/4 deathtouch; other Assassins you
/// control get +1/+1; a player losing after an Assassin of yours attacked them
/// this turn wins you the game.
pub fn ramses_assassin_lord() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        static_abilities: vec![static_ab(
            "Other Assassins you control get +1/+1.",
            StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(assassin()).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerLeftGame, EventScope::AnyPlayer),
            // CR 603.4 — the intervening "if" is read as the trigger resolves.
            effect: Effect::If {
                cond: Predicate::AttackedByYourAssassinThisTurn { who: PlayerRef::TriggerEventPlayer },
                then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..human_assassin("Ramses, Assassin Lord", cost(&[generic(2), u(), b()]), 4, 4)
    }
}

/// Ratonhnhaké꞉ton — {W}{U}{B} 3/3. Until it has dealt damage it has hexproof
/// and can't be blocked; connecting makes a 1/1 Assassin with menace, and an
/// Equipment card from your graveyard returns attached to it.
pub fn ratonhnhake_ton() -> CardDefinition {
    let fresh = || Predicate::Not(Box::new(Predicate::SourceHasDealtDamage));
    let while_fresh = |keyword: Keyword| StaticEffect::WhileCondition {
        condition: fresh(),
        inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword }),
    };
    CardDefinition {
        static_abilities: vec![
            static_ab("As long as it hasn't dealt damage yet, it has hexproof.", while_fresh(Keyword::Hexproof)),
            static_ab("As long as it hasn't dealt damage yet, it can't be blocked.", while_fresh(Keyword::Unblockable)),
        ],
        triggered_abilities: vec![on_hit(Effect::Seq(vec![
            make(1, assassin_token()),
            Effect::Reflexive {
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(
                            R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::InYourGraveyard),
                        ),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::Attach { what: Selector::LastMoved, to: Selector::LastCreatedToken },
                ])),
            },
        ]))],
        ..human_assassin("Ratonhnhaké꞉ton", cost(&[w(), u(), b()]), 3, 3)
    }
}

/// Shao Jun — {1}{U}{R} 3/3; flying and first strike during your turn. Tap two
/// untapped artifacts you control: 1 damage to each opponent.
pub fn shao_jun() -> CardDefinition {
    let yours_turn = |keyword: Keyword| StaticEffect::WhileYourTurn {
        inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword }),
    };
    CardDefinition {
        static_abilities: vec![
            static_ab("During your turn, Shao Jun has flying.", yours_turn(Keyword::Flying)),
            static_ab("During your turn, Shao Jun has first strike.", yours_turn(Keyword::FirstStrike)),
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Artifact.and(R::Untapped), 2)),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            ..Default::default()
        }],
        ..human_assassin("Shao Jun", cost(&[generic(1), u(), r()]), 3, 3)
    }
}

/// Bleeding Effect — {2}{W}{B} Enchantment. Beginning of combat on your turn,
/// your creatures gain each listed keyword a creature card in your graveyard
/// has, until end of turn.
pub fn bleeding_effect() -> CardDefinition {
    let kws = [
        Keyword::Flying,
        Keyword::FirstStrike,
        Keyword::DoubleStrike,
        Keyword::Deathtouch,
        Keyword::Hexproof,
        Keyword::Indestructible,
        Keyword::Lifelink,
        Keyword::Menace,
        Keyword::Reach,
        Keyword::Trample,
        Keyword::Vigilance,
    ];
    let mine = || Selector::EachMatching { zone: ZoneRef::Battlefield, filter: R::Creature.and(R::ControlledByYou) };
    CardDefinition {
        name: "Bleeding Effect",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(
                kws.into_iter()
                    .map(|k| Effect::If {
                        cond: Predicate::ValueAtLeast(
                            Value::CardsInGraveyardMatching {
                                who: PlayerRef::You,
                                filter: R::Creature.and(R::HasKeyword(k.clone())),
                            },
                            Value::ONE,
                        ),
                        then: Box::new(until_eot(mine(), k)),
                        else_: Box::new(Effect::Noop),
                    })
                    .collect(),
            ),
        }],
        ..Default::default()
    }
}

/// Rooftop Bypass — {1}{U}{B} Enchantment. Nontoken creatures of yours
/// connecting make a 1/1 Assassin with menace (once per batch).
pub fn rooftop_bypass() -> CardDefinition {
    CardDefinition {
        name: "Rooftop Bypass",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![yours_hit(R::Creature.and(R::NotToken), make(1, assassin_token()))],
        ..Default::default()
    }
}

/// The Revelations of Ezio — {2}{B} Saga. I: destroy a tapped creature an
/// opponent controls. II: your attacking Assassins get +1/+1 counters this
/// turn. III: an Assassin creature card returns with an extra +1/+1 counter.
pub fn the_revelations_of_ezio() -> CardDefinition {
    CardDefinition {
        name: "The Revelations of Ezio",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, Effect::Destroy { what: target_filtered(R::Creature.and(R::Tapped).and(R::ControlledByOpponent)) }),
            (
                2,
                Effect::GrantTriggeredAbilityThisTurnToMatching {
                    filter: R::Creature.and(assassin()).and(R::ControlledByYou),
                    trigger: Box::new(on_attack(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    })),
                },
            ),
            (
                3,
                Effect::Seq(vec![
                    Effect::SpellEntersWithCounters { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::Move {
                        what: target_filtered(R::Creature.and(assassin()).and(R::InYourGraveyard)),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

// ── Tovolar, Dire Overlord (RG) ──────────────────────────────────────────────

fn wolfish() -> R {
    R::HasCreatureType(CreatureType::Wolf).or(R::HasCreatureType(CreatureType::Werewolf))
}

fn your_wolfish() -> R {
    R::Creature.and(wolfish()).and(R::ControlledByYou)
}

fn wolf_token() -> TokenDefinition {
    token("Wolf", 2, 2, Color::Green, vec![CreatureType::Wolf])
}

/// A night face: a Werewolf creature with the day face's colors (CR 105.2c).
fn night(name: &'static str, day_cost: &ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Werewolf], ..Default::default() },
        color_indicator: day_cost.colors(),
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// CR 702.145 — a daybound front over a nightbound back.
fn daybound(mut front: CardDefinition, mut back: CardDefinition) -> CardDefinition {
    front.keywords.push(Keyword::Daybound);
    back.keywords.push(Keyword::Nightbound);
    CardDefinition { back_face: Some(Box::new(back)), ..front }
}

/// The classic werewolf pair: no spells last turn flips it, two or more
/// flips it back.
fn classic_werewolf(mut front: CardDefinition, mut back: CardDefinition) -> CardDefinition {
    front.triggered_abilities.push(crate::effect::shortcut::werewolf_day_transform());
    back.triggered_abilities.push(crate::effect::shortcut::werewolf_night_transform());
    CardDefinition { back_face: Some(Box::new(back)), ..front }
}

fn human_werewolf(name: &'static str, mana: ManaCost, more: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    let mut types = vec![CreatureType::Human];
    types.extend(more);
    types.push(CreatureType::Werewolf);
    creature(name, mana, types, p, t)
}

/// "Whenever this creature transforms into [this face]".
fn on_transform(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource), effect }
}

fn anthem(filter: R, power: i32, toughness: i32, keywords: Vec<Keyword>) -> StaticEffect {
    StaticEffect::AnthemForFilter {
        filter,
        power,
        toughness,
        keywords,
        opponents: false,
        all_players: false,
        only_your_turn: false,
        scale_by_counters_on_self: None,
    }
}

fn other_wolfish_get(p: i32, t: i32) -> StaticAbility {
    static_ab(
        "Other Wolves and Werewolves you control get the bonus.",
        anthem(R::Creature.and(wolfish()).and(R::OtherThanSource), p, t, vec![]),
    )
}

fn attacking_get(p: i32, keywords: Vec<Keyword>) -> StaticAbility {
    static_ab("Attacking creatures you control get the bonus.", anthem(R::Creature.and(R::IsAttacking), p, 0, keywords))
}

fn begin_your_combat(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
        effect,
    }
}

fn wolf_draw() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).dealt_by(R::Creature.and(wolfish())),
        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
    }
}

/// Tovolar, Dire Overlord // Tovolar, the Midnight Scourge — {1}{R}{G} 3/3
/// legendary Human Werewolf, daybound. A Wolf or Werewolf of yours connecting
/// draws; your upkeep with three or more makes it night and transforms any
/// number of your Human Werewolves. Night: 4/4, {X}{R}{G}: +X/+0 and trample.
pub fn tovolar_dire_overlord() -> CardDefinition {
    let pack = || Predicate::SelectorCountAtLeast { sel: Selector::EachPermanent(your_wolfish()), n: Value::Const(3) };
    let front = CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            wolf_draw(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl).with_filter(pack()),
                effect: Effect::If {
                    cond: pack(),
                    then: Box::new(Effect::Seq(vec![
                        Effect::BecomeNight,
                        Effect::ForEachChosen {
                            from: Selector::EachPermanent(
                                R::Creature
                                    .and(R::HasCreatureType(CreatureType::Human))
                                    .and(R::HasCreatureType(CreatureType::Werewolf))
                                    .and(R::ControlledByYou),
                            ),
                            body: Box::new(Effect::Transform { what: Selector::TriggerSource }),
                            headless_takes: Some(R::Creature),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..human_werewolf("Tovolar, Dire Overlord", cost(&[generic(1), r(), g()]), vec![], 3, 3)
    };
    let pumped = || target_filtered(R::Creature.and(wolfish()).and(R::ControlledByYou));
    let back = CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![wolf_draw()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), r(), g()]),
            effect: Effect::Seq(vec![
                pump(pumped(), Value::XFromCost, Value::ZERO),
                until_eot(Selector::Target(0), Keyword::Trample),
            ]),
            ..Default::default()
        }],
        ..night("Tovolar, the Midnight Scourge", &cost(&[generic(1), r(), g()]), 4, 4)
    };
    daybound(front, back)
}

/// The Celestus — {3} legendary Artifact. Day as it enters if neither; {T}:
/// any color; {3}, {T}: flip day and night (sorcery speed); each flip gains
/// you 1 and you may loot.
pub fn the_celestus() -> CardDefinition {
    CardDefinition {
        name: "The Celestus",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        as_enters_effect: Some(Effect::If {
            cond: Predicate::Not(Box::new(Predicate::Any(vec![Predicate::IsDay, Predicate::IsNight]))),
            then: Box::new(Effect::BecomeDay),
            else_: Box::new(Effect::Noop),
        }),
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                mana_cost: cost(&[generic(3)]),
                effect: Effect::If {
                    cond: Predicate::IsNight,
                    then: Box::new(Effect::BecomeDay),
                    else_: Box::new(Effect::BecomeNight),
                },
                ..Default::default()
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DayNightChanged, EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
                Effect::MayDo {
                    description: "The Celestus: draw a card, then discard a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    ])),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Avabruck Caretaker // Hollowhenge Huntmaster — {4}{G}{G} 4/4 hexproof,
/// daybound: two +1/+1 counters on another creature of yours each combat.
/// Night: 6/6 hexproof, your other permanents have hexproof, two counters on
/// each creature you control.
pub fn avabruck_caretaker() -> CardDefinition {
    let mana = cost(&[generic(4), g(), g()]);
    let front = CardDefinition {
        keywords: vec![Keyword::Hexproof],
        triggered_abilities: vec![begin_your_combat(Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        ..human_werewolf("Avabruck Caretaker", mana.clone(), vec![], 4, 4)
    };
    let back = CardDefinition {
        keywords: vec![Keyword::Hexproof],
        static_abilities: vec![static_ab(
            "Other permanents you control have hexproof.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::ControlledByYou.and(R::OtherThanSource)),
                keyword: Keyword::Hexproof,
            },
        )],
        triggered_abilities: vec![begin_your_combat(Effect::AddCounter {
            what: Selector::EachMatching { zone: ZoneRef::Battlefield, filter: R::Creature.and(R::ControlledByYou) },
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        ..night("Hollowhenge Huntmaster", &mana, 6, 6)
    };
    daybound(front, back)
}

/// Breakneck Rider // Neck Breaker — {1}{R}{R} 3/3; night 4/3, attacking
/// creatures you control get +1/+0 and trample.
pub fn breakneck_rider() -> CardDefinition {
    let mana = cost(&[generic(1), r(), r()]);
    classic_werewolf(
        human_werewolf("Breakneck Rider", mana.clone(), vec![CreatureType::Scout], 3, 3),
        CardDefinition {
            static_abilities: vec![attacking_get(1, vec![Keyword::Trample])],
            ..night("Neck Breaker", &mana, 4, 3)
        },
    )
}

/// Child of the Pack // Savage Packmate — {2}{R}{G} 2/5, {2}{R}{G}: a 2/2
/// Wolf; night 5/5 trample, other creatures you control get +1/+0.
pub fn child_of_the_pack() -> CardDefinition {
    let mana = cost(&[generic(2), r(), g()]);
    daybound(
        CardDefinition {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2), r(), g()]),
                effect: make(1, wolf_token()),
                ..Default::default()
            }],
            ..human_werewolf("Child of the Pack", mana.clone(), vec![], 2, 5)
        },
        CardDefinition {
            keywords: vec![Keyword::Trample],
            static_abilities: vec![static_ab(
                "Other creatures you control get +1/+0.",
                anthem(R::Creature.and(R::OtherThanSource), 1, 0, vec![]),
            )],
            ..night("Savage Packmate", &mana, 5, 5)
        },
    )
}

/// Cult of the Waxing Moon — {4}{G} 5/4 Human Shaman. A permanent of yours
/// transforming into a non-Human creature makes a 2/2 Wolf.
pub fn cult_of_the_waxing_moon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Transformed, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Human)))),
            }),
            effect: make(1, wolf_token()),
        }],
        ..creature("Cult of the Waxing Moon", cost(&[generic(4), g()]), vec![CreatureType::Human, CreatureType::Shaman], 5, 4)
    }
}

/// Duskwatch Recruiter // Krallenhorde Howler — {1}{G} 2/2, {2}{G}: a creature
/// from the top three; night 3/3, your creature spells cost {1} less.
pub fn duskwatch_recruiter() -> CardDefinition {
    let mana = cost(&[generic(1), g()]);
    classic_werewolf(
        CardDefinition {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2), g()]),
                effect: Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                    pick_filter: Some(R::Creature),
                    optional: true,
                    ..Default::default()
                })),
                ..Default::default()
            }],
            ..human_werewolf("Duskwatch Recruiter", mana.clone(), vec![CreatureType::Warrior], 2, 2)
        },
        CardDefinition {
            static_abilities: vec![static_ab(
                "Creature spells you cast cost {1} less to cast.",
                StaticEffect::CostReduction { filter: R::Creature, amount: 1 },
            )],
            ..night("Krallenhorde Howler", &mana, 3, 3)
        },
    )
}

/// Hermit of the Natterknolls // Lone Wolf of the Natterknolls — {2}{G} 2/3:
/// an opponent's spell on your turn draws you one (night 3/5: two).
pub fn hermit_of_the_natterknolls() -> CardDefinition {
    let mana = cost(&[generic(2), g()]);
    let punish = |n: i32| TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(Predicate::IsTurnOf(PlayerRef::You)),
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(n) },
    };
    classic_werewolf(
        CardDefinition {
            triggered_abilities: vec![punish(1)],
            ..human_werewolf("Hermit of the Natterknolls", mana.clone(), vec![], 2, 3)
        },
        CardDefinition { triggered_abilities: vec![punish(2)], ..night("Lone Wolf of the Natterknolls", &mana, 3, 5) },
    )
}

/// Hound Tamer // Untamed Pup — {2}{G} 3/3 trample, {3}{G}: a +1/+1 counter;
/// night 4/4, your other Wolves and Werewolves have trample.
pub fn hound_tamer() -> CardDefinition {
    let mana = cost(&[generic(2), g()]);
    let grow = || ActivatedAbility {
        mana_cost: cost(&[generic(3), g()]),
        effect: Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        ..Default::default()
    };
    daybound(
        CardDefinition {
            keywords: vec![Keyword::Trample],
            activated_abilities: vec![grow()],
            ..human_werewolf("Hound Tamer", mana.clone(), vec![], 3, 3)
        },
        CardDefinition {
            keywords: vec![Keyword::Trample],
            static_abilities: vec![static_ab(
                "Other Wolves and Werewolves you control have trample.",
                StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(your_wolfish().and(R::OtherThanSource)),
                    keyword: Keyword::Trample,
                },
            )],
            activated_abilities: vec![grow()],
            ..night("Untamed Pup", &mana, 4, 4)
        },
    )
}

/// Howlpack Piper // Wildsong Howler — {3}{G} 2/2, can't be countered;
/// {1}{G}, {T}: a creature from hand onto the battlefield, untapping it for a
/// Wolf or Werewolf (sorcery speed). Night 4/4: entering or transforming,
/// a creature from the top six.
pub fn howlpack_piper() -> CardDefinition {
    let mana = cost(&[generic(3), g()]);
    let dig = || {
        Effect::LookPickToHand(Box::new(crate::effect::LookPick {
            who: PlayerRef::You,
            count: Value::Const(6),
            pick_filter: Some(R::Creature),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        }))
    };
    daybound(
        CardDefinition {
            keywords: vec![Keyword::CantBeCountered],
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                mana_cost: cost(&[generic(1), g()]),
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Creature,
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: Some(Box::new(Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::LastMoved, filter: wolfish() },
                        then: Box::new(Effect::Untap { what: Selector::This, up_to: None }),
                        else_: Box::new(Effect::Noop),
                    })),
                },
                ..Default::default()
            }],
            ..human_werewolf("Howlpack Piper", mana.clone(), vec![], 2, 2)
        },
        CardDefinition { triggered_abilities: vec![etb(dig()), on_transform(dig())], ..night("Wildsong Howler", &mana, 4, 4) },
    )
}

/// Huntmaster of the Fells // Ravager of the Fells — {2}{R}{G} 2/2: entering
/// or transforming in, a 2/2 Wolf and 2 life. Night 4/4 trample: transforming
/// in, 2 to an opponent or planeswalker and 2 to up to one creature of theirs.
pub fn huntmaster_of_the_fells() -> CardDefinition {
    let mana = cost(&[generic(2), r(), g()]);
    let howl = || Effect::Seq(vec![make(1, wolf_token()), Effect::GainLife { who: Selector::You, amount: Value::Const(2) }]);
    let ravage = Effect::OptionalTargets {
        min: 1,
        body: Box::new(Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::OpponentPlayer.or(R::Planeswalker)), amount: Value::Const(2) },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::SameControllerAsTargetSlot(0)) },
                amount: Value::Const(2),
            },
        ])),
    };
    classic_werewolf(
        CardDefinition {
            triggered_abilities: vec![etb(howl()), on_transform(howl())],
            ..human_werewolf("Huntmaster of the Fells", mana.clone(), vec![], 2, 2)
        },
        CardDefinition {
            keywords: vec![Keyword::Trample],
            triggered_abilities: vec![on_transform(ravage)],
            ..night("Ravager of the Fells", &mana, 4, 4)
        },
    )
}

/// Ill-Tempered Loner // Howlpack Avenger — {2}{R}{R} 3/3: damage dealt to it
/// is dealt again to any target; {1}{R}: +2/+0. Night 4/4: damage to any
/// permanent of yours.
pub fn ill_tempered_loner() -> CardDefinition {
    let mana = cost(&[generic(2), r(), r()]);
    let firebreathe = || ActivatedAbility {
        mana_cost: cost(&[generic(1), r()]),
        effect: pump(Selector::This, Value::Const(2), Value::ZERO),
        ..Default::default()
    };
    let retort = || Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::TriggerEventAmount };
    daybound(
        CardDefinition {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                effect: retort(),
            }],
            activated_abilities: vec![firebreathe()],
            ..human_werewolf("Ill-Tempered Loner", mana.clone(), vec![], 3, 3)
        },
        CardDefinition {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Permanent.and(R::ControlledByYou),
                }),
                effect: retort(),
            }],
            activated_abilities: vec![firebreathe()],
            ..night("Howlpack Avenger", &mana, 4, 4)
        },
    )
}

/// Immerwolf — {1}{R}{G} 2/2 Wolf, intimidate. Your other Wolves and
/// Werewolves get +1/+1; your non-Human Werewolves can't transform.
pub fn immerwolf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Intimidate],
        static_abilities: vec![
            other_wolfish_get(1, 1),
            static_ab(
                "Non-Human Werewolves you control can't transform.",
                StaticEffect::CantTransform {
                    filter: R::HasCreatureType(CreatureType::Werewolf)
                        .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Human))))
                        .and(R::ControlledByYou),
                },
            ),
        ],
        ..creature("Immerwolf", cost(&[generic(1), r(), g()]), vec![CreatureType::Wolf], 2, 2)
    }
}

/// Instigator Gang // Wildblood Pack — {3}{R} 2/3, attacking creatures you
/// control get +1/+0; night 5/5 trample, +3/+0.
pub fn instigator_gang() -> CardDefinition {
    let mana = cost(&[generic(3), r()]);
    classic_werewolf(
        CardDefinition { static_abilities: vec![attacking_get(1, vec![])], ..human_werewolf("Instigator Gang", mana.clone(), vec![], 2, 3) },
        CardDefinition {
            keywords: vec![Keyword::Trample],
            static_abilities: vec![attacking_get(3, vec![])],
            ..night("Wildblood Pack", &mana, 5, 5)
        },
    )
}

/// Kessig Naturalist // Lord of the Ulvenwald — {R}{G} 2/2: attacking, add {R}
/// or {G} that lasts the turn. Night 3/3: also your other Wolves and Werewolves
/// get +1/+1.
pub fn kessig_naturalist() -> CardDefinition {
    let mana = cost(&[r(), g()]);
    let rush = || {
        on_attack(Effect::AddManaKeptThisTurnOfColors {
            who: PlayerRef::You,
            colors: vec![Color::Red, Color::Green],
            amount: Value::ONE,
        })
    };
    daybound(
        CardDefinition { triggered_abilities: vec![rush()], ..human_werewolf("Kessig Naturalist", mana.clone(), vec![], 2, 2) },
        CardDefinition {
            static_abilities: vec![other_wolfish_get(1, 1)],
            triggered_abilities: vec![rush()],
            ..night("Lord of the Ulvenwald", &mana, 3, 3)
        },
    )
}

/// Nightpack Ambusher — {2}{G}{G} 4/4 Wolf, flash. Your other Wolves and
/// Werewolves get +1/+1; an end step you cast nothing in makes a 2/2 Wolf.
pub fn nightpack_ambusher() -> CardDefinition {
    let quiet = || Predicate::Not(Box::new(Predicate::SpellsCastThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE }));
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![other_wolfish_get(1, 1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(quiet()),
            effect: Effect::If { cond: quiet(), then: Box::new(make(1, wolf_token())), else_: Box::new(Effect::Noop) },
        }],
        ..creature("Nightpack Ambusher", cost(&[generic(2), g(), g()]), vec![CreatureType::Wolf], 4, 4)
    }
}

/// Sage of Ancient Lore // Werewolf of Ancient Hunger — {4}{G} */* (your hand
/// size); entering, draw. Night: vigilance, trample, */* every hand together.
pub fn sage_of_ancient_lore() -> CardDefinition {
    let mana = cost(&[generic(4), g()]);
    classic_werewolf(
        CardDefinition {
            dynamic_pt: Some(crate::card::DynamicPt::ControllerHandSize),
            triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
            ..human_werewolf("Sage of Ancient Lore", mana.clone(), vec![CreatureType::Shaman], 0, 0)
        },
        CardDefinition {
            keywords: vec![Keyword::Vigilance, Keyword::Trample],
            dynamic_pt: Some(crate::card::DynamicPt::AllPlayersHandTotal),
            ..night("Werewolf of Ancient Hunger", &mana, 0, 0)
        },
    )
}

/// Scorned Villager // Moonscarred Werewolf — {1}{G} 1/1, {T}: {G}; night 2/2
/// vigilance, {T}: {G}{G}.
pub fn scorned_villager() -> CardDefinition {
    let mana = cost(&[generic(1), g()]);
    classic_werewolf(
        CardDefinition {
            activated_abilities: vec![crate::sets::tap_add(Color::Green)],
            ..human_werewolf("Scorned Villager", mana.clone(), vec![], 1, 1)
        },
        CardDefinition {
            keywords: vec![Keyword::Vigilance],
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green, Color::Green]) },
                ..Default::default()
            }],
            ..night("Moonscarred Werewolf", &mana, 2, 2)
        },
    )
}

/// Silverfur Partisan — {2}{G} 2/2 Wolf Warrior, trample. A Wolf or Werewolf
/// of yours becoming the target of an instant or sorcery spell makes a Wolf.
pub fn silverfur_partisan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YourCreatureTargeted)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: wolfish() })
                .caused_by(R::IsSpellOnStack.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))),
            effect: make(1, wolf_token()),
        }],
        ..creature("Silverfur Partisan", cost(&[generic(2), g()]), vec![CreatureType::Wolf, CreatureType::Warrior], 2, 2)
    }
}

/// Ulrich of the Krallenhorde // Ulrich, Uncontested Alpha — {3}{R}{G} 4/4
/// legendary: entering or transforming in, a creature gets +4/+4. Night 6/6:
/// transforming in, it may fight a non-Werewolf creature you don't control.
pub fn ulrich_of_the_krallenhorde() -> CardDefinition {
    let mana = cost(&[generic(3), r(), g()]);
    let rouse = || pump(target_filtered(R::Creature), Value::Const(4), Value::Const(4));
    classic_werewolf(
        CardDefinition {
            supertypes: vec![Supertype::Legendary],
            triggered_abilities: vec![etb(rouse()), on_transform(rouse())],
            ..human_werewolf("Ulrich of the Krallenhorde", mana.clone(), vec![], 4, 4)
        },
        CardDefinition {
            supertypes: vec![Supertype::Legendary],
            triggered_abilities: vec![on_transform(Effect::MayDo {
                description: "Ulrich fights?".into(),
                body: Box::new(Effect::Fight {
                    attacker: Selector::This,
                    defender: target_filtered(
                        R::Creature
                            .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Werewolf))))
                            .and(R::Not(Box::new(R::ControlledByYou))),
                    ),
                }),
            })],
            ..night("Ulrich, Uncontested Alpha", &mana, 6, 6)
        },
    )
}

/// Volatile Arsonist // Dire-Strain Anarchist — {3}{R}{R} 4/4 menace, haste:
/// attacking, 1 damage to each of up to one creature, player and planeswalker
/// (night 5/5: 2).
pub fn volatile_arsonist() -> CardDefinition {
    let mana = cost(&[generic(3), r(), r()]);
    let burn = |n: i32| {
        on_attack(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::DealDamage { to: Selector::TargetFiltered { slot: 0, filter: R::Creature }, amount: Value::Const(n) },
                Effect::DealDamage { to: Selector::TargetFiltered { slot: 1, filter: R::Player }, amount: Value::Const(n) },
                Effect::DealDamage { to: Selector::TargetFiltered { slot: 2, filter: R::Planeswalker }, amount: Value::Const(n) },
            ])),
        })
    };
    daybound(
        CardDefinition {
            keywords: vec![Keyword::Menace, Keyword::Haste],
            triggered_abilities: vec![burn(1)],
            ..human_werewolf("Volatile Arsonist", mana.clone(), vec![], 4, 4)
        },
        CardDefinition {
            keywords: vec![Keyword::Menace, Keyword::Haste],
            triggered_abilities: vec![burn(2)],
            ..night("Dire-Strain Anarchist", &mana, 5, 5)
        },
    )
}

/// Weaver of Blossoms // Blossom-Clad Werewolf — {2}{G} 2/3, {T}: any color;
/// night 3/4, {T}: two mana of any one color.
pub fn weaver_of_blossoms() -> CardDefinition {
    let mana = cost(&[generic(2), g()]);
    let tap_for = |n: i32| ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(n)) },
        ..Default::default()
    };
    daybound(
        CardDefinition { activated_abilities: vec![tap_for(1)], ..human_werewolf("Weaver of Blossoms", mana.clone(), vec![], 2, 3) },
        CardDefinition { activated_abilities: vec![tap_for(2)], ..night("Blossom-Clad Werewolf", &mana, 3, 4) },
    )
}

/// Full Moon's Rise — {1}{G} Enchantment. Your Werewolves get +1/+0 and
/// trample; sacrifice it: regenerate them all.
pub fn full_moons_rise() -> CardDefinition {
    let werewolves = || R::Creature.and(R::HasCreatureType(CreatureType::Werewolf));
    CardDefinition {
        name: "Full Moon's Rise",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![static_ab(
            "Werewolf creatures you control get +1/+0 and have trample.",
            anthem(werewolves(), 1, 0, vec![Keyword::Trample]),
        )],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Regenerate { what: Selector::EachPermanent(werewolves().and(R::ControlledByYou)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Howlpack Resurgence — {2}{G} Enchantment, flash. Your Wolves and Werewolves
/// get +1/+1 and trample.
pub fn howlpack_resurgence() -> CardDefinition {
    CardDefinition {
        name: "Howlpack Resurgence",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        static_abilities: vec![static_ab(
            "Each creature you control that's a Wolf or a Werewolf gets +1/+1 and has trample.",
            anthem(R::Creature.and(wolfish()), 1, 1, vec![Keyword::Trample]),
        )],
        ..Default::default()
    }
}

/// Ranger Class — {1}{G} Class. Entering, a 2/2 Wolf. Level 2 ({1}{G}): your
/// attacks put a +1/+1 counter on an attacker. Level 3 ({3}{G}): look at and
/// cast creature spells from the top of your library.
pub fn ranger_class() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    let at_three = |effect: StaticEffect| StaticEffect::WhileClassLevelAtLeast { n: 3, inner: Box::new(effect) };
    CardDefinition {
        name: "Ranger Class",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        triggered_abilities: vec![
            etb(make(1, wolf_token())),
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                    .with_filter(Predicate::SourceClassLevelAtLeast(2)),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::IsAttacking).and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        static_abilities: vec![
            static_ab("You may look at the top card of your library any time.", at_three(StaticEffect::MayLookAtOwnLibraryTop)),
            static_ab(
                "You may cast creature spells from the top of your library.",
                at_three(StaticEffect::PlayFromLibraryTop { filter: R::Creature }),
            ),
        ],
        activated_abilities: vec![level_up(cost(&[generic(1), g()]), 1), level_up(cost(&[generic(3), g()]), 2)],
        ..Default::default()
    }
}

/// Moonlight Hunt — {1}{G} Instant. Each Wolf and Werewolf of yours deals
/// damage equal to its power to target creature you don't control.
pub fn moonlight_hunt() -> CardDefinition {
    spell(
        "Moonlight Hunt",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::EachDealsDamageEqualToPower {
            dealers: Selector::EachPermanent(your_wolfish()),
            target: target_filtered(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
        },
    )
}

/// Waxing Moon — {1}{G} Instant. Transform up to one Werewolf of yours; your
/// creatures gain trample until end of turn.
pub fn waxing_moon() -> CardDefinition {
    spell(
        "Waxing Moon",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::HasCreatureType(CreatureType::Werewolf)).and(R::ControlledByYou),
                effect: Box::new(Effect::Transform { what: Selector::Target(0) }),
            },
            until_eot(
                Selector::EachMatching { zone: ZoneRef::Battlefield, filter: R::Creature.and(R::ControlledByYou) },
                Keyword::Trample,
            ),
        ]),
    )
}

/// Unnatural Moonrise — {R}{G} Sorcery, flashback {2}{R}{G}. It becomes night;
/// a creature gets +1/+0, trample and "connecting draws a card" this turn.
pub fn unnatural_moonrise() -> CardDefinition {
    let it = || target_filtered(R::Creature);
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(2), r(), g()]))],
        ..spell(
            "Unnatural Moonrise",
            cost(&[r(), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::BecomeNight,
                pump(it(), Value::ONE, Value::ZERO),
                until_eot(it(), Keyword::Trample),
                Effect::GrantTriggeredAbility {
                    what: it(),
                    trigger: Box::new(on_hit(Effect::Draw { who: Selector::You, amount: Value::ONE })),
                    duration: Duration::EndOfTurn,
                },
            ]),
        )
    }
}
