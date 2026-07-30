//! Mystical Archive (STA) — the Strixhaven companion set of reprinted
//! instants and sorceries with new art. These ride existing engine
//! primitives (X-draw, kicker, additional-cost sacrifice, reveal-until-find,
//! impulse-exile). Grouped here rather than scattered across the reprint
//! homes so the Strixhaven draft/cube pools can pull them as one slice.

use crate::card::{
    AdditionalCastCost, AlternativeCost, CardDefinition, CardType, Effect, Keyword, Predicate,
    SelectionRequirement, Selector, Value, Zone,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, RevealMissDest, ZoneDest, ZoneRef};
use crate::mana::{Color, b, cost, g, generic, r, u, x};

// ── Infuriate ────────────────────────────────────────────────────────────────

/// Infuriate — {R} Instant. Target creature gets +3/+2 until end of turn.
pub fn infuriate() -> CardDefinition {
    CardDefinition {
        name: "Infuriate",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(3),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── Blue Sun's Zenith ─────────────────────────────────────────────────────────

/// Blue Sun's Zenith — {X}{U}{U}{U} Instant. Target player draws X cards.
/// Shuffle Blue Sun's Zenith into its owner's library.
pub fn blue_suns_zenith() -> CardDefinition {
    CardDefinition {
        name: "Blue Sun's Zenith",
        cost: cost(&[x(), u(), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::XFromCost,
            },
            Effect::ShuffleSelfIntoLibrary,
        ]),
        ..Default::default()
    }
}

// ── Abundant Harvest ──────────────────────────────────────────────────────────

/// Abundant Harvest — {G} Sorcery. Choose land or nonland. Reveal from the top
/// of your library until you reveal a card of the chosen kind; put it into your
/// hand and the rest on the bottom in a random order.
///
/// Modeled as a `ChooseN` over two `RevealUntilFind` arms (misses bottomed).
pub fn abundant_harvest() -> CardDefinition {
    let dig = |find: SelectionRequirement| Effect::RevealUntilFind {
        who: PlayerRef::You,
        find,
        to: ZoneDest::Hand(PlayerRef::You),
        // No printed reveal limit; `cap` is bounded by the library size at
        // resolution, so a large constant means "dig the whole library".
        cap: Value::Const(250),
        life_per_revealed: 0,
        miss_dest: RevealMissDest::BottomRandom,
    };
    CardDefinition {
        name: "Abundant Harvest",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![0],
            modes: vec![
                dig(SelectionRequirement::Land),
                dig(SelectionRequirement::Nonland),
            ],
        },
        ..Default::default()
    }
}

// ── Urza's Rage ───────────────────────────────────────────────────────────────

/// Urza's Rage — {2}{R} Instant. Kicker {8}{R}. Can't be countered. Deals 3
/// damage to any target; if kicked, deals 10 instead (and that damage can't be
/// prevented).
pub fn urzas_rage() -> CardDefinition {
    CardDefinition {
        name: "Urza's Rage",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![
            Keyword::CantBeCountered,
            Keyword::Kicker(cost(&[generic(8), r()])),
        ],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(10),
            }),
            else_: Box::new(Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(3),
            }),
        },
        ..Default::default()
    }
}

// ── Natural Order ─────────────────────────────────────────────────────────────

/// Natural Order — {2}{G}{G} Sorcery. Additional cost: sacrifice a green
/// creature. Search your library for a green creature card, put it onto the
/// battlefield, then shuffle.
pub fn natural_order() -> CardDefinition {
    CardDefinition {
        name: "Natural Order",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Creature
                .and(SelectionRequirement::HasColor(Color::Green)),
            count: 1,
        }],
        effect: Effect::Search {
            who: PlayerRef::You,
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
            filter: SelectionRequirement::Creature
                .and(SelectionRequirement::HasColor(Color::Green)),
        },
        ..Default::default()
    }
}

/// Tainted Pact — {1}{B} Instant. Exile the top card of your library; you may
/// put it into your hand unless it shares a name with a card already exiled
/// this way, which ends the process. Repeat until you take a card or hit a
/// duplicate name.
pub fn tainted_pact() -> CardDefinition {
    CardDefinition {
        name: "Tainted Pact",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ExileUntilDuplicateName {
            who: PlayerRef::You,
        },
        ..Default::default()
    }
}

/// Mizzix's Mastery — {3}{R} Sorcery. Exile target instant/sorcery from your
/// graveyard and cast a copy of it for free. Overload {5}{R}{R}{R}: do that
/// for each instant/sorcery in your graveyard.
///
/// Faithful: the exiled card stays in exile and a COPY is cast for free
/// (CR 707.12, `CastWithoutPayingImmediate { copy: true }`); under overload
/// each card is exiled and its copy free-cast in turn.
pub fn mizzixs_mastery() -> CardDefinition {
    let is_filter = SelectionRequirement::HasCardType(CardType::Instant)
        .or(SelectionRequirement::HasCardType(CardType::Sorcery));
    // "You may cast a COPY of the exiled card" (CR 707.12): the original
    // stays in exile; the copy is cast without paying its mana cost.
    let free_cast = |what| Effect::CastWithoutPayingImmediate {
        what,
        source_zone: Zone::Exile,
        exile_after: false,
        copy: true,
    };
    CardDefinition {
        name: "Mizzix's Mastery",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(is_filter.clone().and(SelectionRequirement::InYourGraveyard)),
                to: ZoneDest::Exile,
            },
            free_cast(Selector::LastMoved),
        ]),
        alternative_cost: Some(AlternativeCost {
            awaken: false,
            mana_cost: cost(&[generic(5), r(), r(), r()]),
            // `ForEach` binds each graveyard card to `TriggerSource`; reference
            // it directly (not `LastMoved`, which accumulates across the loop).
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachMatching {
                    zone: ZoneRef::Graveyard(PlayerRef::You),
                    filter: is_filter.clone(),
                },
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Exile,
                    },
                    free_cast(Selector::TriggerSource),
                ])),
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}
