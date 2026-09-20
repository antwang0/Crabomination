//! The most-built **commanders** the catalog was missing —
//! `COMMANDER_BACKLOG.md`'s section 1 rather than its staples list.
//!
//! The split from `sets::cmdr` is the one that file's header draws: `cmdr.rs`
//! is for cards whose printed text is *about* the format (it reads the command
//! zone, a commander, or a colour identity). A most-built commander is an
//! ordinary legendary creature; what makes it a commander is the deck it
//! leads, not its text.
//!
//! ⚠ The theme here is narrower than "commanders": these are the ones whose
//! text is about the **whole table**, so they do something different at two
//! seats than at eight, and they are worth having for the pod runner even
//! before a deck is built around them.
//!
//! ⚠⚠ **Check the catalog before writing one.** Ayara, First of Locthwain was
//! drafted for this module and dropped: the concurrent session had already
//! shipped it in `recent330` while this file was being written, and the
//! generated `COMMANDER_BACKLOG.md` a batch is sized from is a snapshot, not
//! a lock. `grep -rn "pub fn <slug>" crabomination_catalog/src` costs one
//! second; the ambiguous-glob error that catches it otherwise costs a build.
//!
//! Tests in `tests/core_rules/commander_cards.rs`, which already holds this
//! branch's multi-seat card tests.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value, WardCost,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, Predicate, PlayerRef, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Nekusar, the Mindrazer — {2}{U}{B}{R} Legendary Creature — Zombie Wizard
/// 2/4. "At the beginning of each player's draw step, that player draws an
/// additional card. Whenever an opponent draws a card, Nekusar deals 1 damage
/// to that player." (EDHREC's 18th most-built commander.)
///
/// Both halves are shapes the catalog already ships: the first is Howling
/// Mine's exactly (`StepBegins(Draw)` scoped to `AnyPlayer`, drawing for the
/// active player — the player whose draw step it is), and the second is
/// Underworld Dreams' exactly (`CardDrawn` scoped to `OpponentControl`,
/// damaging `PlayerRef::Triggerer`).
///
/// 📐 It is the clearest card in the catalog for what a seat count does to a
/// board: at two seats it draws one opponent an extra card a turn and pings
/// them once, and at eight it does that to seven players — the symmetric
/// half scales with the table and the damage half only hits opponents.
pub fn nekusar_the_mindrazer() -> CardDefinition {
    CardDefinition {
        name: "Nekusar, the Mindrazer",
        cost: cost(&[generic(2), u(), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Draw),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Kenrith, the Returned King — {4}{W} Legendary Creature — Human Noble 5/5,
/// with five activated abilities and nothing else. (EDHREC's 20th most-built
/// commander, and the highest-ranked one this module could reach.)
///
/// ⚠ **Every clause reads the whole table and none of them says "you".**
/// "**All** creatures gain trample and haste" is every seat's, not the
/// controller's; "**target player** gains 5 life" and "target player draws a
/// card" can aim at an opponent, which is what makes Kenrith a group-hug
/// commander rather than a five-colour value engine; and "put target creature
/// card from **a** graveyard onto the battlefield **under its owner's
/// control**" is two table-facing halves at once — any graveyard, and the
/// creature goes to whoever owns it rather than to Kenrith's controller.
///
/// No new primitive: `Selector::EachPermanent` without a controller scope is
/// "all creatures", `PlayerRef::Target(0)` is the printed "target player", and
/// `ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved }` is the
/// owner's-control clause.
pub fn kenrith_the_returned_king() -> CardDefinition {
    let ability = |mana, effect| ActivatedAbility { mana_cost: mana, effect, ..Default::default() };
    CardDefinition {
        name: "Kenrith, the Returned King",
        cost: cost(&[generic(4), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        activated_abilities: vec![
            ability(
                cost(&[r()]),
                Effect::GrantKeywords {
                    what: Selector::EachPermanent(R::Creature),
                    keywords: vec![Keyword::Trample, Keyword::Haste],
                    duration: Duration::EndOfTurn,
                },
            ),
            ability(
                cost(&[generic(1), g()]),
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ),
            ability(
                cost(&[generic(2), w()]),
                Effect::GainLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(5),
                },
            ),
            ability(
                cost(&[generic(3), u()]),
                Effect::Draw {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                },
            ),
            ability(
                cost(&[generic(4), b()]),
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InGraveyard)),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOfMoved,
                        tapped: false,
                    },
                },
            ),
        ],
        ..Default::default()
    }
}

/// Zedruu the Greathearted — {1}{U}{R}{W} Legendary Creature — Minotaur Monk
/// 2/4. "At the beginning of your upkeep, you gain X life and draw X cards,
/// where X is the number of permanents you own that your opponents control.
/// {U}{R}{W}: Target opponent gains control of target permanent you control."
///
/// The archetypal group-hug commander, and the one whose text cannot be
/// written at all without the distinction between **owning** and
/// **controlling** a permanent: X counts `OwnedByYou ∧ ControlledByOpponent`,
/// a set that is empty in every game where nobody has given anything away.
///
/// The activated ability is Donate's effect with Donate's two target slots —
/// slot 0 the opponent, slot 1 the permanent — so `GainControl { to:
/// Some(PlayerRef::Target(0)) }` over a `TargetFiltered { slot: 1 }`. ⚠ The
/// duration is `Permanent`: the printed card gives the permanent away for
/// good, which is what makes X grow rather than reset.
pub fn zedruu_the_greathearted() -> CardDefinition {
    let given_away = || {
        Value::count(Selector::EachPermanent(
            R::OwnedByYou.and(R::ControlledByOpponent),
        ))
    };
    CardDefinition {
        name: "Zedruu the Greathearted",
        cost: cost(&[generic(1), u(), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: given_away() },
                Effect::Draw { who: Selector::You, amount: given_away() },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), r(), w()]),
            effect: Effect::GainControl {
                what: Selector::TargetFiltered { slot: 1, filter: R::ControlledByYou },
                to: Some(PlayerRef::Target(0)),
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Phelddagrif — {1}{G}{W}{U} Legendary Creature — Phelddagrif 4/4. Three
/// activated abilities, each of which pairs a small benefit for its
/// controller with a **gift to an opponent**:
///
/// * `{G}`: trample until end of turn. Target opponent creates a 1/1 green
///   Hippo creature token.
/// * `{W}`: flying until end of turn. Target opponent gains 2 life.
/// * `{U}`: return Phelddagrif to its owner's hand. Target opponent **may**
///   draw a card.
///
/// The original group-hug card, and the one that shows the three shapes a
/// gift takes: a token the *opponent* creates (`CreateToken { who:
/// Target(0) }`), life the opponent gains, and a draw the opponent **chooses**
/// — `Effect::MayDoBy`, because the printed "may" belongs to the opponent and
/// not to the activating player. A plain `MayDo` would ask the wrong seat.
pub fn phelddagrif() -> CardDefinition {
    let hippo = || TokenDefinition {
        name: "Hippo".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hippo],
            ..Default::default()
        },
        ..Default::default()
    };
    let ability = |mana, effect| ActivatedAbility { mana_cost: mana, effect, ..Default::default() };
    let self_keyword = |kw| Effect::GrantKeyword {
        what: Selector::This,
        keyword: kw,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Phelddagrif",
        cost: cost(&[generic(1), g(), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phelddagrif],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        activated_abilities: vec![
            ability(
                cost(&[g()]),
                Effect::Seq(vec![
                    self_keyword(Keyword::Trample),
                    Effect::CreateToken {
                        who: PlayerRef::Target(0),
                        count: Value::ONE,
                        definition: std::sync::Arc::new(hippo()),
                    },
                ]),
            ),
            ability(
                cost(&[w()]),
                Effect::Seq(vec![
                    self_keyword(Keyword::Flying),
                    Effect::GainLife {
                        who: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::Const(2),
                    },
                ]),
            ),
            ability(
                cost(&[u()]),
                Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    },
                    Effect::MayDoBy {
                        who: PlayerRef::Target(0),
                        description: "Draw a card?".into(),
                        body: Box::new(Effect::Draw {
                            who: Selector::Player(PlayerRef::Target(0)),
                            amount: Value::ONE,
                        }),
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Miirym, Sentinel Wyrm — {3}{G}{U}{R} Legendary Creature — Dragon Spirit
/// 6/6. "Flying, ward {2}. Whenever another **nontoken** Dragon you control
/// enters, create a token that's a copy of it, except the token isn't
/// legendary." (EDHREC's 28th most-built commander.)
///
/// ⚠ Two riders on the trigger and both are load-bearing. **Nontoken** is
/// what stops the copy from copying itself into an unbounded loop — the token
/// it makes is a Dragon you control entering, and without `R::IsToken.negate()`
/// the engine would be right to fire again forever. **Except the token isn't
/// legendary** is `non_legendary: true`: a legendary copy would meet its
/// original under CR 704.5j and one of the two would be put into the
/// graveyard immediately, which is the whole card undone.
///
/// `Selector::TriggerSource` is the Dragon that entered, so the copy is of
/// the right object without a target slot.
pub fn miirym_sentinel_wyrm() -> CardDefinition {
    CardDefinition {
        name: "Miirym, Sentinel Wyrm",
        cost: cost(&[generic(3), g(), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Spirit],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Dragon)
                        .and(R::OtherThanSource)
                        .and(R::IsToken.negate()),
                }),
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::TriggerSource,
                extra_creature_types: Vec::new(),
                extra_card_types: Vec::new(),
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: true,
                legendary: false,
                extra_keywords: Vec::new(),
            },
        }],
        ..Default::default()
    }
}
