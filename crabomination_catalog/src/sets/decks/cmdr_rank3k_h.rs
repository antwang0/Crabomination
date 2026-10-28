//! Commander: one-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), eighth file. Tests in `tests/recent_b/cmdr_rank3k_h.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_filtered, unearth};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
use std::sync::Arc;

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
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches {
        what: Selector::TriggerSource,
        filter,
    }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn draw(n: Value) -> Effect {
    Effect::Draw {
        who: Selector::You,
        amount: n,
    }
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: n,
        definition: Arc::new(def),
    }
}

fn permanent(name: &'static str, mana: ManaCost, kind: CardType) -> CardDefinition {
    spell(name, mana, kind, Effect::Noop)
}

// ── Lobelia, Defender of Bag End ─────────────────────────────────────────────

/// No One Left Behind — {4}{B} sorcery, {3} less if it targets a creature card
/// with mana value 3 or less. Return target creature card from your graveyard
/// to the battlefield.
pub fn no_one_left_behind() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_if_target: Some((R::Creature.and(R::ManaValueAtMost(3)), 3)),
        ..spell(
            "No One Left Behind",
            cost(&[generic(4), b()]),
            CardType::Sorcery,
            Effect::Move {
                what: target_filtered(R::Creature.from_your_graveyard()),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        )
    }
}

// ── Kuro, Pitlord ────────────────────────────────────────────────────────────

/// The Darkhold — {1}{B}{B} legendary Book artifact. {T}: a sin counter on it;
/// draw a card per sin counter, then lose 1 life per card in your hand.
/// {6}{B}{B}, {T}: each player's life total becomes the lowest among all
/// players; sorcery speed.
pub fn the_darkhold() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Book],
            ..Default::default()
        },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Sin,
                        amount: Value::ONE,
                    },
                    draw(Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Sin,
                    }),
                    Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::HandSizeOf(PlayerRef::You),
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(6), b(), b()]),
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::SetLifeTotal {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::LowestLifeTotal,
                },
                ..Default::default()
            },
        ],
        ..permanent(
            "The Darkhold",
            cost(&[generic(1), b(), b()]),
            CardType::Artifact,
        )
    }
}

// ── Rhuk, Hexgold Nabber / Barret Wallace ────────────────────────────────────

/// Reckless Crew — {3}{R} sorcery. Create X 2/1 red Dwarf Berserkers, X the
/// Vehicles plus Equipment you control; for each, you may attach an Equipment
/// you control to it (one already on a creature may move).
pub fn reckless_crew() -> CardDefinition {
    let count = Value::PermanentCountControlledByMatching(
        PlayerRef::You,
        R::HasArtifactSubtype(ArtifactSubtype::Vehicle)
            .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
    );
    let dwarf = TokenDefinition {
        name: "Dwarf Berserker".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Berserker],
            ..Default::default()
        },
        ..Default::default()
    };
    spell(
        "Reckless Crew",
        cost(&[generic(3), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            make(count, dwarf),
            Effect::ForEach {
                selector: Selector::LastCreatedTokens,
                body: Box::new(Effect::MayDo {
                    description: "Attach an Equipment you control to this Dwarf?".into(),
                    // "An Equipment you control" — one already on another
                    // creature may be moved to the Dwarf (CR 701.3b).
                    body: Box::new(Effect::Attach {
                        what: Selector::one_of(yours(R::HasArtifactSubtype(ArtifactSubtype::Equipment))),
                        to: Selector::TriggerSource,
                    }),
                }),
            },
        ]),
    )
}

// ── Mishra, Excavation Prodigy ───────────────────────────────────────────────

/// Mishra's Research Desk — {1} artifact. {1}, {T}, sacrifice it: exile the top
/// two cards of your library; until the end of your next turn you may play
/// one of them. Unearth {1}{R}.
pub fn mishras_research_desk() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_cost: true,
                effect: crate::sets::impulse_choose_one(2, crate::card::MayPlayDuration::EndOfControllersNextTurn),
                ..Default::default()
            },
            unearth(cost(&[generic(1), r()])),
        ],
        ..permanent(
            "Mishra's Research Desk",
            cost(&[generic(1)]),
            CardType::Artifact,
        )
    }
}

// ── Koth of the Homestead / Elena, Turk Recruit ──────────────────────────────

/// Roads Go Ever, Ever On — {1}{W} Saga. I: search for up to two basic Plains,
/// exile them, shuffle; gain 2 life. II, III: put a card exiled with it into
/// its owner's hand. IV: whenever you attack this turn, target creature you
/// control gets +1/+1 per Plains you control until end of turn.
pub fn roads_go_ever_ever_on() -> CardDefinition {
    let plains = || R::IsBasicLand.and(R::HasLandType(LandType::Plains));
    let search = || Effect::Search {
        who: PlayerRef::You,
        filter: plains(),
        to: ZoneDest::ExileWithSourceStamp,
    };
    let fetch_back = || Effect::Move {
        what: Selector::one_of(Selector::CardExiledWithSource),
        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
    };
    let per_plains = || {
        Value::PermanentCountControlledByMatching(PlayerRef::You, R::HasLandType(LandType::Plains))
    };
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    search(),
                    search(),
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Const(2),
                    },
                ]),
            ),
            (2, fetch_back()),
            (3, fetch_back()),
            (
                4,
                Effect::TriggersThisTurn {
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                        effect: Effect::PumpPT {
                            what: target_filtered(R::Creature.and(R::ControlledByYou)),
                            power: per_plains(),
                            toughness: per_plains(),
                            duration: Duration::EndOfTurn,
                        },
                    }],
                },
            ),
        ],
        ..permanent(
            "Roads Go Ever, Ever On",
            cost(&[generic(1), w()]),
            CardType::Enchantment,
        )
    }
}

// ── Melira, the Living Cure ──────────────────────────────────────────────────

/// Ajani, Sleeper Agent — {1}{G}{G/W/P}{W} planeswalker, loyalty 4,
/// compleated. +1: reveal the top card; a creature or planeswalker card goes
/// to your hand. −3: distribute three +1/+1 counters
/// among up to three target creatures; they gain vigilance until end of turn.
/// −6: an emblem — whenever you cast a creature or planeswalker spell, target
/// opponent gets two poison counters. A missed reveal may go to the bottom.
pub fn ajani_sleeper_agent() -> CardDefinition {
    let loyalty = |cost: i32, effect: Effect| crate::card::LoyaltyAbility {
        loyalty_cost: cost,
        effect,
        ..Default::default()
    };
    let cast_cw = || R::HasCardType(CardType::Creature).or(R::HasCardType(CardType::Planeswalker));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Ajani],
            ..Default::default()
        },
        keywords: vec![Keyword::Compleated],
        base_loyalty: 4,
        loyalty_abilities: vec![
            loyalty(
                1,
                Effect::Seq(vec![
                    Effect::RevealTopOfLibrary { who: PlayerRef::You },
                    Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                            filter: cast_cw(),
                        },
                        then: Box::new(Effect::Move {
                            what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                            to: ZoneDest::Hand(PlayerRef::You),
                        }),
                        else_: Box::new(Effect::MayDo {
                            description: "Put the revealed card on the bottom of your library?".into(),
                            body: Box::new(Effect::Move {
                                what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                                to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Bottom },
                            }),
                        }),
                    },
                ]),
            ),
            loyalty(
                -3,
                Effect::Seq(vec![
                    Effect::DistributeCounters {
                        total: Value::Const(3),
                        counter: CounterType::PlusOnePlusOne,
                        filter: R::Creature,
                        max_targets: 3,
                    },
                    Effect::GrantKeyword {
                        what: Selector::AllTargets,
                        keyword: Keyword::Vigilance,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            ),
            loyalty(
                -6,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Ajani, Sleeper Agent".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                            .with_filter(trigger_is(cast_cw())),
                        effect: Effect::AddPoison {
                            who: target_filtered(R::Player.and(R::ControlledByOpponent)),
                            amount: Value::Const(2),
                        },
                    }],
                    statics: vec![],
                },
            ),
        ],
        ..permanent(
            "Ajani, Sleeper Agent",
            cost(&[
                generic(1),
                g(),
                crate::mana::phyrexian_hybrid(Color::Green, Color::White),
                w(),
            ]),
            CardType::Planeswalker,
        )
    }
}

// ── Barrin, Master Wizard ────────────────────────────────────────────────────

/// Magus of the Unseen — {1}{U} 1/1 Human Wizard. {1}{U}, {T}: untap target
/// artifact an opponent controls and gain control of it until end of turn; it
/// gains haste. When you lose control of it, tap it (Ray of Command's shape).
pub fn magus_of_the_unseen() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Untap {
                    what: target_filtered(R::Artifact.and(R::ControlledByOpponent)),
                    up_to: None,
                },
                Effect::GainControl {
                    what: Selector::Target(0),
                    to: None,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::DelayUntil {
                    kind: crate::effect::DelayedTriggerKind::WhenYouLoseControlOfTarget,
                    body: Box::new(Effect::Tap { what: Selector::Target(0) }),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Magus of the Unseen",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}
