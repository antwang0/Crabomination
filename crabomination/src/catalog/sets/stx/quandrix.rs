//! Quandrix (G/U) college cards from Strixhaven.
//!
//! Quandrix cares about **Fractal tokens** (0/0 green-and-blue with
//! variable +1/+1 counters), spell-cast triggers, and X-cost scaling.
//! The first-pass set here covers the two college "Apprentice" /
//! "Pledgemage" creatures plus a couple of mono-flavour scaling cards.
//! Larger Fractal-creator effects (Body of Research, Fractal Anomaly)
//! are already wired in `mono` / SOS — those compose against the same
//! `LastCreatedToken` plumbing this module re-uses.

use super::no_abilities;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, Keyword,
    Selector, SelectionRequirement, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::{magecraft, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{cost, generic, g, u, Color};

// ── Quandrix Apprentice ─────────────────────────────────────────────────────

/// Quandrix Apprentice — {G}{U}, 1/1 Elf Druid.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// target creature you control gets +1/+1 until end of turn."
///
/// Same shape as Eager First-Year (the Silverquill apprentice), just
/// gated to a creature you control rather than any creature. Wired via
/// the new `effect::shortcut::magecraft` helper plus
/// `Predicate::EntityMatches` on the cast.
pub fn quandrix_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Apprentice",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Quandrix Pledgemage ─────────────────────────────────────────────────────

/// Quandrix Pledgemage — {1}{G}{U}, 2/2 Fractal Wizard. "{1}{G}{U}: Put
/// a +1/+1 counter on Quandrix Pledgemage."
///
/// Pure activated +1/+1 counter pump. The Fractal subtype is already in
/// the engine (added with the SOS Fractal package), so the body and
/// counter accrual are faithful to the printed card.
pub fn quandrix_pledgemage() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pledgemage",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1), g(), u()]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Decisive Denial ─────────────────────────────────────────────────────────

/// Decisive Denial — {G}{U} Instant. "Choose one — / • Counter target
/// noncreature spell unless its controller pays {2}. / • Target creature
/// you control deals damage equal to its power to target creature you
/// don't control."
///
/// 🟡 Push: both modes wired. Mode 0: counter-noncreature-unless-{2}.
/// Mode 1: one-sided "deal damage equal to power" — slot 0 is the
/// user-picked friendly creature (used as the damage-source value via
/// `Value::PowerOf(Target(0))`); the opp creature is auto-picked via
/// `Selector::one_of(EachPermanent(opp creature))` since the engine
/// doesn't yet support a multi-target prompt (slot 1 wouldn't be
/// populated by the cast). Same approximation as Chelonian Tackle's
/// "fights up to one target creature an opponent controls". The
/// damage is one-sided (printed "deals damage to", not "fights"), so
/// the friendly creature doesn't take return damage like in `Effect::
/// Fight`.
pub fn decisive_denial() -> CardDefinition {
    use crate::mana::{ManaCost, generic as gen_pip};
    let two = ManaCost { symbols: vec![gen_pip(2)] };
    CardDefinition {
        name: "Decisive Denial",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: counter target noncreature spell unless its controller
            // pays {2}.
            Effect::CounterUnlessPaid {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasCardType(CardType::Creature).negate()),
                ),
                mana_cost: two,
            },
            // Mode 1: target creature you control deals damage equal to
            // its power to a creature you don't control. The friendly
            // creature is the user-picked slot 0 (filtered via the
            // primary-target check on cast); the opp creature is auto-
            // picked via `Selector::one_of(EachPermanent(opp creature))`
            // — same approximation as Chelonian Tackle's "fights up to
            // one target opp creature".
            Effect::DealDamage {
                to: Selector::one_of(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                )),
                amount: Value::PowerOf(Box::new(target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ))),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Quandrix Fractal token ──────────────────────────────────────────────────

/// 0/0 green-and-blue Fractal creature token. Mirrors the SOS catalog's
/// `fractal_token()` (the Strixhaven 2021 set predates the SOS catalog
/// but uses the same token definition). Pulled out into a helper so STX
/// 2021 Quandrix cards (Tend the Pests, Snow Day) can reuse it without
/// each card carrying its own copy of the token shape.
fn quandrix_fractal_token() -> TokenDefinition {
    TokenDefinition {
        name: "Fractal".into(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::Blue],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    }
}

// ── Snow Day ────────────────────────────────────────────────────────────────

/// Snow Day — {1}{G}{U} Instant. "Create a 0/0 green and blue Fractal
/// creature token. Put X +1/+1 counters on it, where X is the number of
/// cards in your hand."
///
/// Wired faithfully via the new `Selector::LastCreatedToken` (push II)
/// + `Value::HandSizeOf(You)` — the Fractal enters at 0/0, then receives
/// hand-size-many +1/+1 counters in a single resolution. With a 7-card
/// hand the Fractal lands as a 7/7 — a respectable on-curve threat.
pub fn snow_day() -> CardDefinition {
    CardDefinition {
        name: "Snow Day",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: quandrix_fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::HandSizeOf(PlayerRef::You),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Mentor's Guidance ───────────────────────────────────────────────────────

/// Mentor's Guidance — {2}{G}{U} Sorcery. "Draw two cards. Then put a
/// +1/+1 counter on target creature you control for each card in your
/// hand."
///
/// Wired faithfully via `Value::HandSizeOf(You)` for the counter scaling
/// — after drawing 2, the post-draw hand size powers the counter
/// distribution onto a single target creature you control. Multi-target
/// + "for each" iteration are collapsed onto a single creature target
/// (printed: "target creature" — single target, not fan-out).
pub fn mentors_guidance() -> CardDefinition {
    CardDefinition {
        name: "Mentor's Guidance",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::HandSizeOf(PlayerRef::You),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Quandrix Command ────────────────────────────────────────────────────────

/// Quandrix Command — {1}{G}{U} Instant.
/// "Choose two —
/// • Counter target activated ability.
/// • Put two +1/+1 counters on target creature.
/// • Put target card from a graveyard on the bottom of its owner's library.
/// • Draw a card."
///
/// Push XXIV: 🟡 — printed "choose two" collapses to "choose one" via
/// `Effect::ChooseMode` (same approximation as Moment of Reckoning,
/// Witherbloom / Lorehold / Prismari Commands). Each mode wired
/// faithfully against existing primitives.
pub fn quandrix_command() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Quandrix Command",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: counter target activated ability.
            Effect::CounterAbility {
                what: target_filtered(SelectionRequirement::Permanent),
            },
            // Mode 1: put two +1/+1 counters on target creature.
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            // Mode 2: gy → bottom of owner's library on a target card.
            // Picked from an opponent's graveyard for the auto-target
            // framework (the printed mode targets any graveyard).
            Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::EachOpponent,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Any,
                    },
                    Value::Const(1),
                ),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: crate::effect::LibraryPosition::Bottom,
                },
            },
            // Mode 3: draw a card.
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}


// ── Augmenter Pugilist ──────────────────────────────────────────────────────

/// Augmenter Pugilist — {3}{G}{G}, 6/6 Human Warrior with Trample.
/// Printed Oracle: "Trample. Activated abilities of creatures cost {2}
/// more to activate."
///
/// Push XXX: 🟡. Big-body Quandrix beater — six power, six toughness,
/// trample, all on a {3}{G}{G} curve. The static "activated abilities of
/// creatures cost {2} more" is omitted (no `StaticEffect::TaxActivated
/// Abilities` primitive yet — same gap as Trinisphere's "minimum cost"
/// flavor in CUBE_FEATURES.md). The body alone is still a respectable
/// 5-mana finisher; the static would round it up to a hate piece against
/// equipment / Walking Ballista / +X infinite combos.
pub fn augmenter_pugilist() -> CardDefinition {
    CardDefinition {
        name: "Augmenter Pugilist",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
