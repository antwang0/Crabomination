//! Secrets of Strixhaven Mystical Archive (SOA) — the 2026 companion set
//! of reprinted instants and sorceries. Most SOA reprints already live in
//! their original-set homes; this module holds the nine that had no prior
//! definition anywhere in the catalog.

use crate::card::{
    AdditionalCastCost, CardDefinition, CardType, CreatureType, Effect, Keyword, LandType,
    SelectionRequirement, Subtypes, TokenDefinition,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    CounteredSpellZone, DelayedTriggerKind, Duration, ManaPayload, PlayerRef, Predicate, Selector,
    Value, ZoneDest,
};
use crate::mana::{Color, b, cost, g, generic, r, w, x};

/// Akroma's Will — {3}{W} Instant. "Choose one. If you control a commander
/// as you cast this spell, you may choose both instead. / • Creatures you
/// control gain flying, vigilance, and double strike until end of turn. /
/// • Creatures you control gain lifelink, indestructible, and protection
/// from each color until end of turn."
///
/// With a commander on your battlefield the "may choose both" upgrade
/// runs via ChooseN (a decider may still under-pick to one mode).
pub fn akromas_will() -> CardDefinition {
    let your_creatures = || {
        Selector::EachPermanent(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        )
    };
    let modes = move || {
        vec![
            Effect::GrantKeywords {
                what: your_creatures(),
                keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::DoubleStrike],
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeywords {
                what: your_creatures(),
                keywords: vec![
                    Keyword::Lifelink,
                    Keyword::Indestructible,
                    Keyword::Protection(Color::White),
                    Keyword::Protection(Color::Blue),
                    Keyword::Protection(Color::Black),
                    Keyword::Protection(Color::Red),
                    Keyword::Protection(Color::Green),
                ],
                duration: Duration::EndOfTurn,
            },
        ]
    };
    CardDefinition {
        name: "Akroma's Will",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::YouControlACommander,
            then: Box::new(Effect::ChooseN {
                picks: vec![0, 1],
                modes: modes(),
            }),
            else_: Box::new(Effect::ChooseMode(modes())),
        },
        ..Default::default()
    }
}

/// Reprieve — {1}{W} Instant. "Return target spell to its owner's hand.
/// Draw a card."
pub fn reprieve() -> CardDefinition {
    CardDefinition {
        name: "Reprieve",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpellToZone {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                zone: CounteredSpellZone::OwnerHand,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Return to the Ranks — {X}{W}{W} Sorcery, Convoke. "Return X target
/// creature cards with mana value 2 or less from your graveyard to the
/// battlefield." The X picks are player-chosen via `Effect::MoveChosen`
/// (`Decision::ChooseCards`; auto decider maximizes).
pub fn return_to_the_ranks() -> CardDefinition {
    use crate::effect::ZoneRef;
    CardDefinition {
        name: "Return to the Ranks",
        cost: cost(&[x(), w(), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Convoke],
        effect: Effect::MoveChosen {
            from: Selector::EachMatching {
                zone: ZoneRef::Graveyard(PlayerRef::You),
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(2)),
            },
            filter: None,
            count: Value::XFromCost,
            up_to: false,
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        ..Default::default()
    }
}

/// Winds of Abandon — {1}{W} Sorcery. "Exile target creature you don't
/// control. For each creature exiled this way, its controller searches
/// their library for a basic land card, puts it onto the battlefield
/// tapped, then shuffles. / Overload {4}{W}{W}."
pub fn winds_of_abandon() -> CardDefinition {
    use crate::card::AlternativeCost;
    let not_yours =
        || SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou.negate());
    // Both the searched library AND the fetched land's controller are the
    // exiled creature's controller (LKI via `ControllerOf`).
    let compensate = |who: PlayerRef| Effect::Search {
        who: who.clone(),
        filter: SelectionRequirement::IsBasicLand,
        to: ZoneDest::Battlefield {
            controller: who,
            tapped: true,
        },
    };
    CardDefinition {
        name: "Winds of Abandon",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        // The compensation search resolves BEFORE the exile so
        // `ControllerOf(Target(0))` still reads the on-battlefield
        // controller (the engine's ControllerOf LKI only covers destroys);
        // the board result is order-independent here.
        effect: Effect::Seq(vec![
            compensate(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
            Effect::Exile {
                what: target_filtered(not_yours()),
            },
        ]),
        alternative_cost: Some(AlternativeCost {
            awaken: false,
            mana_cost: cost(&[generic(4), w(), w()]),
            // Overload: "each creature you don't control".
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachPermanent(not_yours()),
                body: Box::new(Effect::Seq(vec![
                    compensate(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                    Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Exile,
                    },
                ])),
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Culling the Weak — {B} Instant. "As an additional cost to cast this
/// spell, sacrifice a creature. / Add {B}{B}{B}{B}."
pub fn culling_the_weak() -> CardDefinition {
    CardDefinition {
        name: "Culling the Weak",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Creature,
            count: 1,
        }],
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColor(Color::Black, Value::Const(4)),
        },
        ..Default::default()
    }
}

/// Subterranean Tremors — {X}{R} Sorcery. "Deals X damage to each
/// creature without flying. If X is 4 or more, destroy all artifacts. If
/// X is 8 or more, create an 8/8 red Lizard creature token."
pub fn subterranean_tremors() -> CardDefinition {
    let x_at_least = |n: i64| Predicate::ValueAtLeast(Value::XFromCost, Value::Const(n as i32));
    CardDefinition {
        name: "Subterranean Tremors",
        cost: cost(&[x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying).negate()),
                ),
                amount: Value::XFromCost,
            },
            Effect::If {
                cond: x_at_least(4),
                then: Box::new(Effect::Destroy {
                    what: Selector::EachPermanent(SelectionRequirement::Artifact),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::If {
                cond: x_at_least(8),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Lizard".to_string(),
                        power: 8,
                        toughness: 8,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Red],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Lizard],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Awaken the Woods — {X}{G}{G} Sorcery. "Create X 1/1 green Forest Dryad
/// land creature tokens."
pub fn awaken_the_woods() -> CardDefinition {
    CardDefinition {
        name: "Awaken the Woods",
        cost: cost(&[x(), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::XFromCost,
            definition: TokenDefinition {
                name: "Forest Dryad".to_string(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Land, CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Dryad],
                    land_types: vec![LandType::Forest],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Berserk — {G} Instant. "Cast this spell only before the combat damage
/// step. / Target creature gains trample and gets +X/+0 until end of
/// turn, where X is its power. At the beginning of the next end step,
/// destroy that creature if it attacked this turn."
///
/// The cast-timing restriction ("only before the combat damage step") is
/// not modeled — cast legality follows normal instant timing.
pub fn berserk() -> CardDefinition {
    CardDefinition {
        name: "Berserk",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::PowerOf(Box::new(Selector::Target(0))),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            // "At the beginning of the next end step, destroy that
            // creature if it attacked this turn." DelayUntil captures
            // Target(0).
            Effect::DelayUntil {
                kind: DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: SelectionRequirement::AttackedThisTurn,
                    },
                    then: Box::new(Effect::Destroy {
                        what: Selector::Target(0),
                    }),
                    else_: Box::new(Effect::Noop),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Glimpse of Nature — {G} Sorcery. "Whenever you cast a creature spell
/// this turn, draw a card." Approximated with the First Day of Class
/// machinery: a draw per creature you control ENTERING this turn (a cast
/// creature spell resolves into an entering creature; token mints also
/// count — the known drift).
pub fn glimpse_of_nature() -> CardDefinition {
    CardDefinition {
        name: "Glimpse of Nature",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreaturesYouControlEnteringThisTurn {
            body: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            }),
        },
        ..Default::default()
    }
}
