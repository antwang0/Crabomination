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
    SelectionRequirement as R, Selector, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w};

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
