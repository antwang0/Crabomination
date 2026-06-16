//! Mystical Archive (STA) — the Strixhaven companion set of reprinted
//! instants and sorceries with new art. These ride existing engine
//! primitives (X-draw, kicker, additional-cost sacrifice, reveal-until-find,
//! impulse-exile). Grouped here rather than scattered across the reprint
//! homes so the Strixhaven draft/cube pools can pull them as one slice.

use crate::card::{
    AdditionalCastCost, CardDefinition, CardType, Effect, Keyword, Predicate, Selector,
    SelectionRequirement, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, RevealMissDest, ZoneDest};
use crate::mana::{cost, g, generic, r, u, x, Color};

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
            modes: vec![dig(SelectionRequirement::Land), dig(SelectionRequirement::Nonland)],
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
        keywords: vec![Keyword::CantBeCountered, Keyword::Kicker(cost(&[generic(8), r()]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(10) }),
            else_: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(3) }),
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
            filter: SelectionRequirement::Creature.and(SelectionRequirement::HasColor(Color::Green)),
            count: 1,
        }],
        effect: Effect::Search {
            who: PlayerRef::You,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            filter: SelectionRequirement::Creature.and(SelectionRequirement::HasColor(Color::Green)),
        },
        ..Default::default()
    }
}
