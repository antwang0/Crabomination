//! Commander: one-card gaps of the 2026-10-14 ranks 1-5500 EDHREC rescan,
//! third file. Tests in `tests/recent_b/cmdr_rescan_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, hybrid, r, u, w};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        keywords,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

/// Flitwing, Lyev Detective — {2}{U} legendary 1/1 Faerie Detective; flying.
/// If you would create one or more tokens, you may create that many Clues
/// instead. (Its own deck.)
pub fn flitwing_lyev_detective() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If you would create one or more tokens, you may create that many Clue tokens instead.",
            effect: StaticEffect::TokensMayBecome { options: vec![crabomination_base::tokens::clue_token()] },
        }],
        ..legend("Flitwing, Lyev Detective", cost(&[generic(2), u()]), vec![CreatureType::Faerie, CreatureType::Detective], 1, 1)
    }
}

/// Venser, Visionary Traveler — {3}{W}{U} legendary planeswalker, loyalty 4;
/// can be your commander. Your nontoken creatures not cast from your hand
/// enter with two extra +1/+1 counters. +1: exile up to one other target
/// permanent you control, returning it at the next end step. −2: for each
/// opponent, return up to one target nonland permanent they control to hand.
pub fn venser_visionary_traveler() -> CardDefinition {
    CardDefinition {
        name: "Venser, Visionary Traveler",
        cost: cost(&[generic(3), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Venser], ..Default::default() },
        base_loyalty: 4,
        can_be_commander: true,
        static_abilities: vec![StaticAbility {
            description: "Each nontoken creature you control that wasn't cast from your hand enters with two additional +1/+1 counters on it.",
            effect: StaticEffect::MatchingEntersWithExtraCounters {
                filter: R::Creature.and(R::NotToken).and(R::ControlledByYou).and(R::Not(Box::new(R::WasCastFromHand))),
                kind: CounterType::PlusOnePlusOne,
                amount: 2,
            },
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Permanent.and(R::ControlledByYou).and(R::OtherThanSource),
                    effect: Box::new(Effect::ExileReturnToOwnerNextEndStep { what: Selector::Target(0), tapped: false }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::ForEachOpponentTarget {
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 8,
                        min_targets: 0,
                        filter: R::Permanent.and(R::Nonland).and(R::ControlledByOpponent),
                        effect: Box::new(Effect::Move {
                            what: Selector::Target(0),
                            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                        }),
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Cheering Crowd — {1}{R/G} 2/2 Human Citizen. At the beginning of each
/// player's first main phase, that player may put a +1/+1 counter on it; if
/// they do, they add {C} for each counter on it. (Tuknir Deathlock.)
pub fn cheering_crowd() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::AnyPlayer),
            effect: Effect::MayDoBy {
                who: PlayerRef::ActivePlayer,
                description: "Put a +1/+1 counter on Cheering Crowd?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Colorless(Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::PlusOnePlusOne,
                        }),
                    },
                ])),
            },
        }],
        ..creature("Cheering Crowd", cost(&[generic(1), hybrid(Color::Red, Color::Green)]), vec![CreatureType::Human, CreatureType::Citizen], 2, 2)
    }
}

/// Unstable Frontier — land. {T}: Add {C}. {T}: target land you control
/// becomes the basic land type of your choice until end of turn. (Loki.)
pub fn unstable_frontier() -> CardDefinition {
    CardDefinition {
        name: "Unstable Frontier",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::LandsBecomeChosenBasicType {
                    what: target_filtered(R::Land.and(R::ControlledByYou)),
                    duration: Duration::EndOfTurn,
                    from_chosen_basic: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Azula, Ruthless Firebender — {2}{B} legendary 3/3 Human Noble;
/// firebending 1. Whenever she attacks, you may discard a card; then you get
/// an experience counter for each player who discarded a card this turn.
/// {2}{B}: +1/+1 per experience counter and menace until end of turn.
pub fn azula_ruthless_firebender() -> CardDefinition {
    let xp_if = |who: PlayerRef| Effect::If {
        cond: Predicate::DiscardedThisTurn { who },
        then: Box::new(Effect::AddExperience(Value::ONE)),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        keywords: vec![Keyword::Firebending(1)],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::MayDiscard {
                description: "Discard a card?".into(),
                count: Value::ONE,
                then: Box::new(Effect::Noop),
                else_: None,
            },
            xp_if(PlayerRef::You),
            Effect::ForEachOpponent { body: Box::new(xp_if(PlayerRef::Triggerer)) },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ControllerExperience,
                    toughness: Value::ControllerExperience,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Menace, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..legend("Azula, Ruthless Firebender", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Noble], 3, 3)
    }
}

/// Nephilim Epochal — {U/R}{R/G}{G/W}{W/B} legendary 4/4 Elder Nephilim. ETB:
/// search for a Nephilim card. Nephilim you control get +1/+1 for each other
/// Nephilim you control and can't be blocked by monocolored creatures.
pub fn nephilim_epochal() -> CardDefinition {
    let nephilim = R::HasCreatureType(CreatureType::Nephilim);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: nephilim.clone(),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        static_abilities: vec![
            StaticAbility {
                description: "Nephilim you control get +1/+1 for each other Nephilim you control.",
                effect: StaticEffect::PumpTeamByControlledPermanents {
                    applies_to: nephilim.clone(),
                    count_filter: nephilim.clone(),
                    per_power: 1,
                    per_toughness: 1,
                    count_graveyard: false,
                    exclude_self: true,
                    per_own_color: false,
                    per_own_counter: None,
                },
            },
            StaticAbility {
                description: "Nephilim you control can't be blocked by monocolored creatures.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Creature.and(nephilim).and(R::ControlledByYou)),
                    keyword: Keyword::CantBeBlockedBy(Box::new(R::Monocolored)),
                },
            },
        ],
        ..legend(
            "Nephilim Epochal",
            cost(&[
                hybrid(Color::Blue, Color::Red),
                hybrid(Color::Red, Color::Green),
                hybrid(Color::Green, Color::White),
                hybrid(Color::White, Color::Black),
            ]),
            vec![CreatureType::Elder, CreatureType::Nephilim],
            4,
            4,
        )
    }
}

/// The Meep — {2}{B} legendary 0/4 Alien; ward—pay 3 life. Whenever it
/// attacks, you may sacrifice another creature; if you do, creatures you
/// control have base P/T X/X until end of turn, X its mana value.
pub fn the_meep() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Life(3))],
        triggered_abilities: vec![on_attack(Effect::MaySacrifice {
            description: "Sacrifice another creature?".into(),
            filter: R::Creature.and(R::OtherThanSource),
            count: Value::ONE,
            then: Box::new(Effect::SetBasePT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::SacrificedManaValue,
                toughness: Value::SacrificedManaValue,
                duration: Duration::EndOfTurn,
            }),
            else_: None,
        })],
        ..legend("The Meep", cost(&[generic(2), b()]), vec![CreatureType::Alien], 0, 4)
    }
}

/// Zagorka, Mother of Sanctum — {1}{G}{U}{R} legendary 4/2 Human Peasant;
/// vigilance, trample. At the beginning of your end step, each player may
/// create a tapped Sanctum land token ("{T}: Add one mana of any color");
/// each opponent who does can't attack you during their next turn.
pub fn zagorka_mother_of_sanctum() -> CardDefinition {
    let sanctum = Arc::new(TokenDefinition {
        name: "Sanctum".into(),
        card_types: vec![CardType::Land],
        tapped: true,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::MayDo {
                    description: "Create a Sanctum?".into(),
                    body: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: sanctum.clone() }),
                },
                Effect::ForEachOpponent {
                    body: Box::new(Effect::AnyPlayerMayAccept {
                        who: PlayerRef::Triggerer,
                        prompt: "Create a Sanctum? You can't attack its owner during your next turn.".into(),
                        accepted: Box::new(Effect::Seq(vec![
                            Effect::CreateToken { who: PlayerRef::AcceptingPlayer, count: Value::ONE, definition: sanctum },
                            Effect::CantAttackYouDuringTheirNextTurn { who: PlayerRef::AcceptingPlayer },
                        ])),
                        otherwise: Box::new(Effect::Noop),
                    }),
                },
            ]),
        }],
        ..legend(
            "Zagorka, Mother of Sanctum",
            cost(&[generic(1), crate::mana::g(), u(), r()]),
            vec![CreatureType::Human, CreatureType::Peasant],
            4,
            2,
        )
    }
}

/// Don & Leo, Problem Solvers — {3}{W/U}{W/U} legendary 4/6 Mutant Ninja
/// Turtle; vigilance. At the beginning of your end step, exile up to one
/// target artifact you control and up to one target creature you control,
/// then return them under their owners' control.
pub fn don_leo_problem_solvers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::ExileAndReturnToOwner {
                    what: Selector::Both(
                        Box::new(Selector::TargetFiltered { slot: 0, filter: R::Artifact.and(R::ControlledByYou) }),
                        Box::new(Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou) }),
                    ),
                }),
            },
        }],
        ..legend(
            "Don & Leo, Problem Solvers",
            cost(&[generic(3), hybrid(Color::White, Color::Blue), hybrid(Color::White, Color::Blue)]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            4,
            6,
        )
    }
}

/// Torrent of Souls — {4}{B/R} sorcery. If {B} was spent, return up to one
/// target creature card from your graveyard to the battlefield; if {R} was
/// spent, creatures target player controls get +2/+0 and haste until end of
/// turn. (Ultimate Green Goblin.)
pub fn torrent_of_souls() -> CardDefinition {
    let theirs = || Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature };
    CardDefinition {
        name: "Torrent of Souls",
        cost: cost(&[generic(4), hybrid(Color::Black, Color::Red)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::ManaSpentOfColorAtLeast { color: Color::Black, at_least: 1 },
                    then: Box::new(Effect::Move {
                        what: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::InYourGraveyard) },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::If {
                    cond: Predicate::ManaSpentOfColorAtLeast { color: Color::Red, at_least: 1 },
                    then: Box::new(Effect::Seq(vec![
                        Effect::PumpPT { what: theirs(), power: Value::Const(2), toughness: Value::ZERO, duration: Duration::EndOfTurn },
                        Effect::GrantKeyword { what: theirs(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        },
        ..Default::default()
    }
}

/// Speedball, New Warrior — {2}{U/R} legendary 2/2 Human Hero. Whenever a
/// player casts a spell that targets him, +2/+2 until end of turn, and you
/// may choose new targets for that spell.
pub fn speedball_new_warrior() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(Predicate::CastSpellTargetsSource),
            effect: Effect::Seq(vec![
                Effect::PumpPT { what: Selector::This, power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
                Effect::MayDo {
                    description: "Choose new targets for that spell?".into(),
                    body: Box::new(Effect::ChooseNewTargetsForSpell { what: Selector::TriggerSource }),
                },
            ]),
        }],
        ..legend("Speedball, New Warrior", cost(&[generic(2), hybrid(Color::Blue, Color::Red)]), vec![CreatureType::Human, CreatureType::Hero], 2, 2)
    }
}

/// Getaway Barrel — {3}{R} artifact. When it is put into a graveyard from
/// the battlefield, reveal the top thirteen cards; put a random creature
/// card among them onto the battlefield and the rest on the bottom in a
/// random order. (Slobad.)
pub fn getaway_barrel() -> CardDefinition {
    CardDefinition {
        name: "Getaway Barrel",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::RandomOf(Box::new(Selector::MatchingAmong {
                        inner: Box::new(Selector::TopOfLibrary { who: PlayerRef::You, count: Value::Const(13) }),
                        filter: R::Creature,
                    })),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::BottomInRandomOrder {
                    what: Selector::TopOfLibrary {
                        who: PlayerRef::You,
                        count: Value::Diff(
                            Box::new(Value::Const(13)),
                            Box::new(Value::CountOf(Box::new(Selector::LastMoved))),
                        ),
                    },
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Doom Foretold — {2}{W}{B} enchantment. At the beginning of each player's
/// upkeep, that player sacrifices a nonland, nontoken permanent; if they
/// can't, they discard a card and lose 2 life, you draw a card, gain 2
/// life, create a 2/2 white Knight with vigilance, then sacrifice this.
/// (Neva.)
pub fn doom_foretold() -> CardDefinition {
    let theirs = R::Nonland.and(R::NotToken);
    let active = || Selector::Player(PlayerRef::ActivePlayer);
    CardDefinition {
        name: "Doom Foretold",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::ControlledBy { who: PlayerRef::ActivePlayer, filter: theirs.clone() }),
                then: Box::new(Effect::Sacrifice { who: active(), count: Value::ONE, filter: theirs }),
                else_: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: active(), amount: Value::ONE, random: false },
                    Effect::LoseLife { who: active(), amount: Value::Const(2) },
                    draw(1),
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(token("Knight", 2, 2, vec![Color::White], vec![CreatureType::Knight], vec![Keyword::Vigilance])),
                    },
                    Effect::SacrificeSource,
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Kuroki, Thief of Talents — {4}{B} legendary 4/4 Spirit; menace. At the
/// beginning of your end step, target opponent may draw four cards; if they
/// do, you may cast a spell from their hand without paying its mana cost;
/// if they don't, two +1/+1 counters on Kuroki.
pub fn kuroki_thief_of_talents() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::AnyPlayerMayAccept {
                who: PlayerRef::Target(0),
                prompt: "Draw four cards?".into(),
                accepted: Box::new(Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::TargetFiltered { slot: 0, filter: R::OpponentPlayer },
                        amount: Value::Const(4),
                    },
                    Effect::CastAnyOrderWithoutPaying {
                        what: Selector::CardsInZone { who: PlayerRef::Target(0), zone: Zone::Hand, filter: R::Any },
                        source_zone: Zone::Hand,
                        filter: None,
                        cap: Some(Value::ONE),
                        total_mana_value: None,
                    },
                ])),
                otherwise: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
            },
        }],
        ..legend("Kuroki, Thief of Talents", cost(&[generic(4), b()]), vec![CreatureType::Spirit], 4, 4)
    }
}

fn exile_top_playable(duration: crate::card::MayPlayDuration) -> Effect {
    Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count: Value::ONE,
        duration,
        pay_any_color: false,
        max_mana_value: None,
        pay_own_cost: true,
        uncast_penalty: None,
    }
}

/// Pick Up the Pace — {1}{R} enchantment. Whenever a creature you control
/// that entered this turn attacks, exile the top card of your library; you
/// may play cards exiled with this as long as you attacked this turn.
/// (Samut, Hazoret's Champion.)
pub fn pick_up_the_pace() -> CardDefinition {
    CardDefinition {
        name: "Pick Up the Pace",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::EnteredThisTurn,
            }),
            effect: exile_top_playable(crate::card::MayPlayDuration::TurnsHolderAttacks { holder: 0 }),
        }],
        ..Default::default()
    }
}

/// Boros Strike-Captain — {1}{R/W}{R/W} 3/3 Minotaur Soldier. Battalion —
/// exile the top card of your library; during any turn you attacked with
/// three or more creatures, you may play that card. (Razia.)
pub fn boros_strike_captain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::battalion(exile_top_playable(
            crate::card::MayPlayDuration::TurnsHolderAttacksWithThreeOrMore { holder: 0 },
        ))],
        ..creature(
            "Boros Strike-Captain",
            cost(&[generic(1), hybrid(Color::Red, Color::White), hybrid(Color::Red, Color::White)]),
            vec![CreatureType::Minotaur, CreatureType::Soldier],
            3,
            3,
        )
    }
}
