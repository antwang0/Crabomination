//! Commander: one-card gaps of the 2026-10-14 ranks 1-5500 EDHREC rescan,
//! third file. Tests in `tests/recent_b/cmdr_rescan_a.rs`.

use crate::card::{
    ActivatedAbility, Adventure, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Selector, ZoneDest};
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
                    per_own_pips: None,
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
                        exile_after: false,
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

/// Ayesha Tanaka, Armorer — {3}{W}{U} legendary 2/4 Human Artificer. Whenever
/// she attacks, look at the top four: put any number of artifact cards with
/// mana value up to her power onto the battlefield tapped, the rest on the
/// bottom in a random order. She can't be blocked while the defending player
/// controls three or more artifacts.
pub fn ayesha_tanaka_armorer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::LookTopPutMatchingOntoBattlefield {
            count: Value::Const(4),
            filter: R::Artifact.and(R::ManaValueAtMostSourcePower),
            then: None,
            max: Some(4),
            tapped: true,
            exile_rest: false,
            rest_to_graveyard: false,
            mandatory: false,
        })],
        static_abilities: vec![StaticAbility {
            description: "Ayesha Tanaka can't be blocked as long as defending player controls three or more artifacts.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Unblockable,
                condition: Predicate::ValueAtLeast(
                    Value::PermanentCountControlledByMatching(PlayerRef::DefendingPlayer, R::Artifact),
                    Value::Const(3),
                ),
            },
        }],
        ..legend("Ayesha Tanaka, Armorer", cost(&[generic(3), w(), u()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 4)
    }
}

/// Maeve, Insidious Singer — {2}{U}{U} legendary 3/4 Siren. {2}{U}: goad
/// target creature; whenever it attacks one of your opponents this turn, you
/// draw a card. (Tetsuo Umezawa.)
pub fn maeve_insidious_singer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Seq(vec![
                Effect::Goad { what: target_filtered(R::Creature) },
                Effect::WhenTargetAttacksOpponentThisTurn { slot: 0, body: Box::new(draw(1)) },
            ]),
            ..Default::default()
        }],
        ..legend("Maeve, Insidious Singer", cost(&[generic(2), u(), u()]), vec![CreatureType::Siren], 3, 4)
    }
}

/// Volcano Hellion — {2}{R}{R} 6/5 Hellion; echo {X}, X your life total. When
/// it enters, it deals an amount of damage of your choice to you and target
/// creature; the damage can't be prevented. (Auntie Blyte.) The choice is
/// offered up to the creature's toughness.
pub fn volcano_hellion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::EchoLifeTotal],
        triggered_abilities: vec![etb(Effect::DamageCantBePrevented {
            body: Box::new(Effect::PlayerChoosesNumber {
                who: Selector::You,
                prompt: "Volcano Hellion: how much damage to you and the creature?".into(),
                max: Value::ToughnessOf(Box::new(Selector::Target(0))),
                then: Box::new(Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ChosenNumber },
                    Effect::DealDamage { to: Selector::You, amount: Value::ChosenNumber },
                ])),
            }),
        })],
        ..creature("Volcano Hellion", cost(&[generic(2), r(), r()]), vec![CreatureType::Hellion], 6, 5)
    }
}

/// Saheeli, Filigree Master — {2}{U}{R} legendary planeswalker, loyalty 3.
/// +1: scry 1, then you may tap an untapped artifact you control; if you do,
/// draw a card. −2: two 1/1 flying Thopter artifact creatures with haste
/// until end of turn. −4: an emblem with "Artifact creatures you control get
/// +1/+1" and "Artifact spells you cast cost {1} less to cast." (Saheeli,
/// Jewel of Avishkar.)
pub fn saheeli_filigree_master() -> CardDefinition {
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Saheeli, Filigree Master",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Saheeli], ..Default::default() },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                    Effect::MayTap {
                        description: "Tap an untapped artifact you control to draw a card?".into(),
                        filter: R::Artifact.and(R::ControlledByYou),
                        count: Value::ONE,
                        then: Box::new(draw(1)),
                        else_: None,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(thopter) },
                    Effect::GrantKeyword {
                        what: Selector::LastCreatedTokens,
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -4,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Saheeli, Filigree Master".into(),
                    triggered: vec![],
                    statics: vec![
                        StaticAbility {
                            description: "Artifact creatures you control get +1/+1.",
                            effect: StaticEffect::PumpPT {
                                applies_to: Selector::EachPermanent(R::Artifact.and(R::Creature).and(R::ControlledByYou)),
                                power: 1,
                                toughness: 1,
                            },
                        },
                        StaticAbility {
                            description: "Artifact spells you cast cost {1} less to cast.",
                            effect: StaticEffect::CostReduction { filter: R::Artifact, amount: 1 },
                        },
                    ],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Dr. Beverly Crusher — {2}{W} legendary 2/2 Human Doctor. If you gaining
/// life causes a triggered ability of a permanent you control to trigger, it
/// triggers an additional time. Whenever you gain life, a +1/+1 counter on
/// her.
pub fn dr_beverly_crusher() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you gaining life causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.",
            effect: StaticEffect::DoubleControllerLifeGainTriggers,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..legend("Dr. Beverly Crusher", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Doctor], 2, 2)
    }
}

/// Tomik, Orzhov Lawmage — {1}{W} legendary 2/1 Human Advisor; flying.
/// Planeswalkers you control have "No more than one creature can attack this
/// planeswalker each combat." {T}: target creature with a +1/+1 counter on
/// it gains flying until end of turn. (Urdnan.)
pub fn tomik_orzhov_lawmage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Planeswalkers you control have \"No more than one creature can attack this planeswalker each combat.\"",
            effect: StaticEffect::YourPlaneswalkersAttackerCap { n: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne))),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend("Tomik, Orzhov Lawmage", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Advisor], 2, 1)
    }
}

/// Hew the Entwood — {3}{R}{R} sorcery. Sacrifice any number of lands, reveal
/// that many, and put any number of the revealed artifact and/or land cards
/// onto the battlefield (lands tapped); the rest go to the bottom at random.
/// Approximation: chosen cards enter one at a time in pick order rather than
/// all nonlands first, then all lands. (Koth, the Geomancer; Marwyn.)
pub fn hew_the_entwood() -> CardDefinition {
    CardDefinition {
        name: "Hew the Entwood",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SacrificeAnyNumber { who: PlayerRef::You, filter: R::Land, per_each: Box::new(Effect::Noop) },
            Effect::LookPickToHand(Box::new(LookPick {
                count: Value::SacrificedCount,
                take: Some(Value::SacrificedCount),
                pick_filter: Some(R::Artifact.or(R::Land)),
                optional: true,
                picked_lands_to_battlefield: true,
                picked_matching_to_battlefield: Some(R::Artifact),
                rest_bottom_random: true,
                ..Default::default()
            })),
        ]),
        ..Default::default()
    }
}

/// Hawkeye, Master Marksman — {1}{R} legendary 2/2 Human Archer Hero; first
/// strike, reach. Whenever he becomes tapped, you may pay {1} up to three
/// times; choose up to that many: target creature can't block this turn / 2
/// damage to target player / discard a card, then draw a card. Approximation:
/// modes are chosen as the trigger resolves, and each targeted mode is its own
/// reflexive trigger.
pub fn hawkeye_master_marksman() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::MayPayGenericUpTo {
                max: Value::Const(3),
                body: Box::new(Effect::ChooseUpToN {
                    max: Box::new(Value::TriggerEventAmount),
                    modes: vec![
                        Effect::Reflexive {
                            body: Box::new(Effect::GrantKeyword {
                                what: target_filtered(R::Creature),
                                keyword: Keyword::CantBlock,
                                duration: Duration::EndOfTurn,
                            }),
                        },
                        Effect::Reflexive {
                            body: Box::new(Effect::DealDamage { to: target_filtered(R::Player), amount: Value::Const(2) }),
                        },
                        Effect::Seq(vec![
                            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                            draw(1),
                        ]),
                    ],
                }),
            },
        }],
        ..legend(
            "Hawkeye, Master Marksman",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Archer, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Zareth San, the Trickster — {3}{U}{B} legendary 4/4 Merfolk Rogue; flash.
/// {2}{U}{B}, return an unblocked attacking Rogue you control to its owner's
/// hand: put this card from your hand onto the battlefield tapped and
/// attacking (a ninjutsu-shaped ability that isn't ninjutsu, so ninjutsu
/// reducers skip it; you choose the defender, CR 508.4). Combat damage to a
/// player: you may put target permanent card from their graveyard onto the
/// battlefield under your control. Approximation: the returned Rogue is
/// auto-picked. (Krydle of Baldur's Gate.)
pub fn zareth_san_the_trickster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), b()]),
            from_hand: true,
            bounce_other_filter: Some((
                R::Creature.and(R::HasCreatureType(CreatureType::Rogue)).and(R::IsAttacking).and(R::IsUnblocked),
                1,
            )),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::JoinCombatAttackingChosen { what: Selector::LastMoved, cleanup: Default::default() },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Put a permanent card from their graveyard onto the battlefield?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::PermanentCard.and(R::InGraveyard).and(R::ControlledByTriggerPlayer)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..legend(
            "Zareth San, the Trickster",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::Merfolk, CreatureType::Rogue],
            4,
            4,
        )
    }
}

/// Fabrication Foundry — {1}{W} artifact. {T}: add {W}, spent only on
/// artifact spells or abilities of artifacts. {2}{W}, {T}, exile one or more
/// other artifacts you control with total mana value X: return target
/// artifact card with mana value X or less from your graveyard to the
/// battlefield; sorcery speed. (The Prydwen, Losheel, Lita.)
pub fn fabrication_foundry() -> CardDefinition {
    CardDefinition {
        name: "Fabrication Foundry",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::White])),
                        crate::mana::SpendRestriction::ArtifactOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                tap_cost: true,
                sorcery_speed: true,
                exile_permanent_cost: Some((R::Artifact.and(R::OtherThanSource), 1)),
                exile_permanent_cost_covers_target_mv: true,
                effect: Effect::Move {
                    what: target_filtered(R::Artifact.from_your_graveyard()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Baron Helmut Zemo — {B}{B}{B} legendary 3/3 Human Noble Villain. Whenever
/// you cast a black spell from your hand, he connives. Boast — exile any
/// number of black cards from your graveyard with fifteen or more black mana
/// symbols among their mana costs: copy those cards; cast up to three of the
/// copies free. A manual seat names the exiled set and the copies it casts;
/// headless takes the fewest cards, then the priciest copies. (The Master of
/// Lake-town, M.O.D.O.K.)
pub fn baron_helmut_zemo() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellMatches(R::HasColor(Color::Black)),
                Predicate::CastSpellFromHand,
            ])),
            effect: crate::effect::shortcut::connive(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            exile_other_filter: Some((R::HasColor(Color::Black), 1)),
            exile_other_total_pips: Some((Color::Black, 15)),
            effect: Effect::CopyCardsAndCastFreeUpTo { what: Selector::CostExiledCards, max: 3 },
            ..crate::effect::shortcut::boast(ManaCost::default(), Effect::Noop)
        }],
        ..legend(
            "Baron Helmut Zemo",
            cost(&[b(), b(), b()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Villain],
            3,
            3,
        )
    }
}

/// Blaster, Combat DJ // Blaster, Morale Booster — {3}{R}{G} legendary 3/3
/// artifact creature Robot; More Than Meets the Eye {1}{R}{G}. Other nontoken
/// artifact creatures and Vehicles you control have modular 1; whenever you
/// put +1/+1 counters on Blaster, convert it. Back: legendary artifact with
/// modular 3 and "{X}, {T}: move X +1/+1 counters from Blaster onto another
/// target artifact; it gains haste; with no counters left, convert Blaster"
/// (sorcery speed). The granted modular is "put into a graveyard", so an
/// uncrewed Vehicle moves its counters too (CR 702.43a). (Balthier and Fran.)
pub fn blaster_combat_dj() -> CardDefinition {
    use crate::card::{AlternativeCost, ArtifactSubtype};
    use crate::mana::{g, x};
    let granted = R::NotToken
        .and(R::OtherThanSource)
        .and(R::ControlledByYou)
        .and(R::Artifact.and(R::Creature).or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)));
    let back = CardDefinition {
        name: "Blaster, Morale Booster",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        color_indicator: vec![Color::Red, Color::Green],
        keywords: vec![Keyword::Modular(3)],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        // CR 702.43a — modular's "put into a graveyard from the battlefield"
        // on a noncreature artifact.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            ..crate::effect::shortcut::modular_dies()
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![x()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::MoveCounters {
                    from: Selector::This,
                    to: target_filtered(R::Artifact.and(R::OtherThanSource)),
                    counter: CounterType::PlusOnePlusOne,
                    amount: Value::XFromCost,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                Effect::If {
                    cond: Predicate::ValueEquals(
                        Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                        Value::Const(0),
                    ),
                    then: Box::new(Effect::Transform { what: Selector::This }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Blaster, Combat DJ",
        cost: cost(&[generic(3), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 3,
        toughness: 3,
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(1), r(), g()]),
            converted: true,
            ..Default::default()
        }),
        static_abilities: vec![
            StaticAbility {
                description: "Other nontoken artifact creatures and Vehicles you control have modular 1.",
                effect: StaticEffect::MatchingEntersWithExtraCounters {
                    filter: granted.clone(),
                    kind: CounterType::PlusOnePlusOne,
                    amount: 1,
                },
            },
            StaticAbility {
                description: "(Modular 1: when they die, their +1/+1 counters may move to an artifact creature.)",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: granted.clone(),
                    ability: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
                        ..crate::effect::shortcut::modular_dies()
                    }),
                },
            },
            StaticAbility {
                description: "(They have modular 1.)",
                effect: StaticEffect::GrantKeyword { applies_to: Selector::EachPermanent(granted), keyword: Keyword::Modular(1) },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::YouPutCounters)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSource }),
            effect: Effect::Transform { what: Selector::This },
        }],
        back_face: Some(Box::new(back)),
        ..Default::default()
    }
}

/// Firesong and Sunspeaker — {4}{R}{W} legendary 4/6 Minotaur Cleric. Red
/// instant and sorcery spells you control have lifelink. Whenever a white
/// instant or sorcery spell causes you to gain life, it deals 3 damage to
/// target creature or player. (Rem Karolus, Stalwart Slayer.)
pub fn firesong_and_sunspeaker() -> CardDefinition {
    let instant_or_sorcery = R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Red instant and sorcery spells you control have lifelink.",
            effect: StaticEffect::YourMatchingInstantSorcerySpellsHaveLifelink { filter: R::HasColor(Color::Red) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Performed(crate::effect::KeywordAct::SpellCausedLifeGain), EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::White).and(instant_or_sorcery),
                },
            ),
            effect: Effect::DealDamage { to: target_filtered(R::Creature.or(R::Player)), amount: Value::Const(3) },
        }],
        ..legend(
            "Firesong and Sunspeaker",
            cost(&[generic(4), r(), w()]),
            vec![CreatureType::Minotaur, CreatureType::Cleric],
            4,
            6,
        )
    }
}

/// Koh, the Face Stealer — {4}{B}{B} legendary 6/6 Shapeshifter Spirit. When
/// Koh enters, exile up to one other target creature. Whenever another
/// nontoken creature dies, you may exile it. Pay 1 life: choose a creature
/// card exiled with Koh; Koh has all activated and triggered abilities of the
/// last chosen card. (Iname as One.)
pub fn koh_the_face_stealer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::ExileLinked { what: target_filtered(R::Creature.and(R::OtherThanSource)) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken.and(R::OtherThanSource) },
                ),
                effect: Effect::MayDo {
                    description: "Exile the creature with Koh?".into(),
                    body: Box::new(Effect::ExileLinked { what: Selector::TriggerSource }),
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            effect: Effect::BorrowAbilitiesOfChosenExiled { filter: R::Creature },
            ..Default::default()
        }],
        ..legend(
            "Koh, the Face Stealer",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Shapeshifter, CreatureType::Spirit],
            6,
            6,
        )
    }
}

/// Asgardian Citadel — land, enters tapped. ETB: gain 1 life. {T}: {R} or {W}.
/// (Thor Odinson.)
pub fn asgardian_citadel() -> CardDefinition {
    super::super::tapland_untyped("Asgardian Citadel", Color::Red, Color::White, super::super::etb_gain_one())
}

/// Lethal Exploit — {1}{B} instant. Target creature gets -2/-2, and an
/// additional -1/-1 for each modified creature you control (CR 700.9).
/// Approximation: the modified creatures are counted as it resolves, not as
/// it's cast. (Exava, Rakdos Blood Witch.)
pub fn lethal_exploit() -> CardDefinition {
    let shrink = || {
        Value::Negate(Box::new(Value::Sum(vec![
            Value::Const(2),
            Value::CountOf(Box::new(Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(R::IsModified),
            ))),
        ])))
    };
    CardDefinition {
        name: "Lethal Exploit",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: shrink(),
            toughness: shrink(),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Detection Tower — land. {T}: add {C}. {1}, {T}: until end of turn, your
/// opponents and creatures your opponents control with hexproof can be the
/// targets of spells and abilities you control as though they didn't have
/// hexproof. (Kumano, Loki, Diaochan.)
pub fn detection_tower() -> CardDefinition {
    CardDefinition {
        name: "Detection Tower",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::IgnoreOpponentHexproofThisTurn,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: chapters,
        ..Default::default()
    }
}

/// Fall of the Impostor — {1}{G}{W} Saga. I, II — a +1/+1 counter on up to
/// one target creature. III — exile a creature with the greatest power among
/// creatures target opponent controls. Approximation: a power tie exiles the
/// first in battlefield order rather than your choice. (Sigurd, Jarl of
/// Ravensthorpe.)
pub fn fall_of_the_impostor() -> CardDefinition {
    use crate::mana::g;
    let pump = Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        }),
    };
    saga(
        "Fall of the Impostor",
        cost(&[generic(1), g(), w()]),
        vec![
            (1, pump.clone()),
            (2, pump),
            (
                3,
                Effect::TargetPlayerThen {
                    filter: R::OpponentPlayer,
                    then: Box::new(Effect::Exile {
                        what: Selector::TakeGreatestPower {
                            inner: Box::new(Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature }),
                            count: Box::new(Value::ONE),
                        },
                    }),
                },
            ),
        ],
    )
}

/// The Phasing of Zhalfir — {2}{U}{U} Saga, read ahead. I, II — another
/// target nonland permanent phases out and can't phase in while this Saga
/// remains. III — destroy all creatures; each one's controller creates a 2/2
/// black Phyrexian per creature of theirs destroyed. (Goldberry,
/// River-Daughter.)
pub fn the_phasing_of_zhalfir() -> CardDefinition {
    // CR 702.26 — "can't phase in for as long as you control this Saga": the
    // marker ends the hold on a control change, the link on its leaving.
    let po = Effect::Seq(vec![
        Effect::PhaseOut { what: target_filtered(R::Nonland.and(R::OtherThanSource)), until_source_leaves: true },
        Effect::HoldPhasedOutWhileYouControlSource,
    ]);
    CardDefinition {
        read_ahead: true,
        ..saga(
            "The Phasing of Zhalfir",
            cost(&[generic(2), u(), u()]),
            vec![
                (1, po.clone()),
                (2, po),
                (
                    3,
                    Effect::Seq(vec![
                        Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
                        Effect::EachPlayerDoes {
                            who: PlayerRef::EachPlayer,
                            body: Box::new(Effect::CreateToken {
                                who: PlayerRef::You,
                                count: Value::CreaturesDestroyedThisResolutionControlledBy(PlayerRef::You),
                                definition: Arc::new(token("Phyrexian", 2, 2, vec![Color::Black], vec![CreatureType::Phyrexian], vec![])),
                            }),
                        },
                    ]),
                ),
            ],
        )
    }
}

/// Spiders-Man, Heroic Horde — {1}{G} legendary 2/3 Spider Hero. Web-slinging
/// {4}{G}{G}. When it enters, if cast using web-slinging, gain 3 life and
/// create two 2/1 green Spiders with reach. (Spider-Ham, Peter Porker.)
pub fn spiders_man_heroic_horde() -> CardDefinition {
    use crate::mana::g;
    CardDefinition {
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(4), g(), g()]),
            return_to_hand: Some((R::Creature.and(R::Tapped), 1)),
            marks_kicked: true,
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: Arc::new(token("Spider", 2, 1, vec![Color::Green], vec![CreatureType::Spider], vec![Keyword::Reach])),
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..legend(
            "Spiders-Man, Heroic Horde",
            cost(&[generic(1), g()]),
            vec![CreatureType::Spider, CreatureType::Hero],
            2,
            3,
        )
    }
}

/// Grim Reaper's Scythe — {2}{B} legendary artifact. Whenever one or more
/// creature cards leave your graveyard, create a 2/2 black Zombie. {3}{B},
/// {T}, sacrifice two creatures: return target creature card from your
/// graveyard to the battlefield with a finality counter (it enters with it,
/// `SpellEntersWithCounters`); sorcery speed. (Egon, God of Death.)
pub fn grim_reapers_scythe() -> CardDefinition {
    CardDefinition {
        name: "Grim Reaper's Scythe",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature })
                .once_per_batch(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(token("Zombie", 2, 2, vec![Color::Black], vec![CreatureType::Zombie], vec![])),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            tap_cost: true,
            sorcery_speed: true,
            sac_other_filter: Some((R::Creature, 2)),
            effect: Effect::Seq(vec![
                Effect::SpellEntersWithCounters { what: Selector::Target(0), kind: CounterType::Finality, amount: Value::ONE },
                Effect::Move {
                    what: target_filtered(R::Creature.from_your_graveyard()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Devout Invocation — {6}{W} sorcery. Tap any number of untapped creatures
/// you control; a 4/4 white flying Angel for each one tapped. (Masako the
/// Humorless.)
pub fn devout_invocation() -> CardDefinition {
    CardDefinition {
        name: "Devout Invocation",
        cost: cost(&[generic(6), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::TapAnyNumberThen {
            filter: R::Creature,
            body: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: Arc::new(token("Angel", 4, 4, vec![Color::White], vec![CreatureType::Angel], vec![Keyword::Flying])),
            }),
        },
        ..Default::default()
    }
}

/// Raiding Schemes — {3}{R}{G} enchantment. Each noncreature spell you cast
/// has conspire. (Wort, the Raidmother.)
pub fn raiding_schemes() -> CardDefinition {
    use crate::mana::g;
    CardDefinition {
        name: "Raiding Schemes",
        cost: cost(&[generic(3), r(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Each noncreature spell you cast has conspire.",
            effect: StaticEffect::GrantConspireToSpells { filter: R::Noncreature, from_exile: false },
        }],
        ..Default::default()
    }
}

/// Spry and Mighty — {4}{G} sorcery. Choose exactly two creatures you
/// control; draw X and they get +X/+X and trample until end of turn, X the
/// difference between their powers. (Tromell, Seymour's Butler.)
pub fn spry_and_mighty() -> CardDefinition {
    use crate::mana::g;
    let pile = || Selector::SeparatedPile { chosen: true };
    let max = Value::PowerOf(Box::new(Selector::TakeGreatestPower { inner: Box::new(pile()), count: Box::new(Value::ONE) }));
    // |p1 − p2| = 2·max − (p1 + p2).
    let x = Value::Diff(Box::new(Value::Sum(vec![max.clone(), max])), Box::new(Value::PowerOf(Box::new(pile()))));
    CardDefinition {
        name: "Spry and Mighty",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseSomeAmong {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            chooser: PlayerRef::You,
            count: Value::Const(2),
            up_to: false,
            other: Box::new(Effect::Noop),
            chosen: Box::new(Effect::WithX {
                x,
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::XFromCost },
                    Effect::PumpPT { what: pile(), power: Value::XFromCost, toughness: Value::XFromCost, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: pile(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                ])),
            }),
        },
        ..Default::default()
    }
}

/// Knowledge Exploitation — {5}{U}{U} Kindred Sorcery — Rogue; prowl {3}{U}.
/// Search target opponent's library for an instant or sorcery card; you may
/// cast it free; then that player shuffles. (Thada Adel, Acquisitor.)
pub fn knowledge_exploitation() -> CardDefinition {
    CardDefinition {
        name: "Knowledge Exploitation",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rogue], ..Default::default() },
        alternative_cost: Some(crate::effect::shortcut::prowl(cost(&[generic(3), u()]), vec![CreatureType::Rogue])),
        effect: Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::Seq(vec![
                Effect::ChooseOneAmong {
                    what: Selector::EachMatching {
                        zone: crate::effect::ZoneRef::Library(PlayerRef::Target(0)),
                        filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    },
                    chooser: PlayerRef::You,
                    other: Box::new(Effect::Noop),
                    chosen: Box::new(Effect::CastWithoutPayingImmediate {
                        what: Selector::SeparatedPile { chosen: true },
                        source_zone: Zone::Library,
                        exile_after: false,
                        copy: false,
                        reduce_generic: 0,
                        pay_own_cost: false,
                    }),
                },
                Effect::ShuffleLibrary { who: PlayerRef::Target(0) },
            ])),
        },
        ..Default::default()
    }
}

/// Azor, the Lawbringer — {2}{W}{W}{U}{U} legendary 6/6 flying Sphinx. When
/// Azor enters, each opponent can't cast instant or sorcery spells during
/// that player's next turn. Whenever Azor attacks, you may pay {X}{W}{U}{U};
/// if you do, gain X life and draw X cards.
pub fn azor_the_lawbringer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::PlayerCantCastMatchingDuringTheirNextTurn {
                who: PlayerRef::EachOpponent,
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            }),
            on_attack(Effect::MayPayXPlus {
                extra: cost(&[w(), u(), u()]),
                description: "Pay {X}{W}{U}{U}: gain X life and draw X".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::XFromCost },
                    Effect::Draw { who: Selector::You, amount: Value::XFromCost },
                ])),
            }),
        ],
        ..legend(
            "Azor, the Lawbringer",
            cost(&[generic(2), w(), w(), u(), u()]),
            vec![CreatureType::Sphinx],
            6,
            6,
        )
    }
}

/// Shah of Naar Isle — {3}{R} 6/6 Efreet; trample, echo {0}. When its echo
/// cost is paid, each opponent may draw up to three cards (three yes/no
/// draws stand in for "up to three"). (Adamaro, First to Desire.)
pub fn shah_of_naar_isle() -> CardDefinition {
    let may_draw = || Effect::MayDo {
        description: "Draw a card?".into(),
        body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
    };
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Echo(ManaCost::default())],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Performed(crate::effect::KeywordAct::EchoPaid), EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSource }),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::EachOpponent,
                body: Box::new(Effect::Seq(vec![may_draw(), may_draw(), may_draw()])),
            },
        }],
        ..creature("Shah of Naar Isle", cost(&[generic(3), r()]), vec![CreatureType::Efreet], 6, 6)
    }
}

/// Go Nuts! — {G} sorcery; teamwork 3 (you may tap creatures you control
/// with total power 3 or more as you cast it). Choose one — or both if
/// teamwork was paid: a +1/+1 counter on target creature; target creature you
/// control fights target creature an opponent controls. Teamwork rides the
/// action-kicker slot. (Bushmaster, Tippy-Toe.)
pub fn go_nuts() -> CardDefinition {
    use crate::mana::g;
    CardDefinition {
        name: "Go Nuts!",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        kicker_action_cost: Some(crate::card::AdditionalCastCost::TapTotalPower {
            filter: R::Creature.and(R::ControlledByYou),
            power: 3,
        }),
        modes_widen: Some(crate::card::ModesWiden::Kicked),
        effect: Effect::ChooseModesCast {
            min: 1,
            max: 1,
            allow_repeats: false,
            modes: vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::Fight {
                    attacker: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                    defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
                },
            ],
        },
        ..Default::default()
    }
}

/// The Irencrag — {2} legendary artifact. {T}: add {C}. Whenever a legendary
/// creature you control enters, you may have it become a legendary Equipment
/// artifact named Everflame, Heroes' Legacy with equip {3} and "Equipped
/// creature gets +3/+3", losing all other abilities. (Yomiji, Adrestia.)
pub fn the_irencrag() -> CardDefinition {
    CardDefinition {
        name: "The Irencrag",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![super::super::tap_add_colorless()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)) },
            ),
            effect: Effect::MayDo {
                description: "Have The Irencrag become Everflame, Heroes' Legacy?".into(),
                body: Box::new(Effect::BecomeEquipment {
                    what: Selector::This,
                    name: "Everflame, Heroes' Legacy",
                    equip: cost(&[generic(3)]),
                    power: 3,
                    toughness: 3,
                }),
            },
        }],
        ..Default::default()
    }
}

/// My Precious // Allure of Power — {3} legendary Equipment: equipped
/// creature has hexproof and can't be blocked; equip—{2}, pay 2 life.
/// Adventure {1}{B} instant: as an additional cost, sacrifice a creature; draw
/// two cards. (Gollum, Riddle Master.)
pub fn my_precious() -> CardDefinition {
    CardDefinition {
        name: "My Precious",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equip_life_cost: 2,
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![Keyword::Hexproof, Keyword::Unblockable],
            ..Default::default()
        }),
        adventure: Some(Box::new(Adventure {
            name: "Allure of Power",
            cost: cost(&[generic(1), b()]),
            card_types: vec![CardType::Instant],
            effect: draw(2),
        })),
        adventure_additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature.and(R::ControlledByYou),
            count: 1,
        }],
        ..Default::default()
    }
}

/// Diaochan, Artful Beauty — {3}{R} legendary 1/1 Human Advisor. {T}: destroy
/// target creature of your choice, then destroy target creature of an
/// opponent's choice; only during your turn, before attackers are declared.
/// (The opponent's pick is made as it resolves, Volcanic Offering's shape.)
pub fn diaochan_artful_beauty() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::Any(vec![
                    Predicate::CurrentStepIs(TurnStep::Upkeep),
                    Predicate::CurrentStepIs(TurnStep::Draw),
                    Predicate::CurrentStepIs(TurnStep::PreCombatMain),
                    Predicate::CurrentStepIs(TurnStep::BeginCombat),
                ]),
            ])),
            effect: Effect::Seq(vec![
                Effect::Destroy { what: Selector::TargetFiltered { slot: 0, filter: R::Creature } },
                Effect::OpponentChoosesPermanentThen {
                    filter: R::Creature,
                    body: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                    chooser: None,
                    theirs: false,
                    untargeted: false,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Diaochan, Artful Beauty",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            1,
            1,
        )
    }
}
