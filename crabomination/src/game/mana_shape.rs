//! The **shape** questions about a printed mana ability: can this estimate
//! count it, how much and which colours could it make, and is its amount a
//! runtime value rather than a constant.
//!
//! They lived in `server::bot`, where `available_mana` is their only caller
//! that *plays*. `game::actions::mana_summary_of` needs the same four
//! answers to pack them per definition (PERF `(-317)`), and `actions.rs` has
//! no dependency on `server::bot` and must not grow one — so the predicates
//! moved down here rather than being written twice. **Two hand-written
//! copies of one predicate is the bug class this branch keeps closing**;
//! there is one of each, and both callers read it.
//!
//! Every function here is a pure function of the ability or effect handed
//! in: no board, no instance, no seat. That is what makes the pack in
//! `mana_summary_of` sound.

use crate::effect::{ActivatedAbility, Effect, ManaPayload};

// ⚠ **`#[inline]` on each of these is load-bearing, and it is the whole cost
// of the move.** `release-fast` is codegen-units 16 with no LTO, so a callee
// in another module is a call rather than an inlined body: leaving them bare
// took `accumulate_mana_colors` from **53,844 Ir to 2,320,396** on a six-game
// `cube` run (+0.10 % of the pool, +0.18 % of `fixed`) purely by moving the
// file. `accumulate_mana_colors_seq` keeps its `#[inline(never)]` — that one
// is `(-136)`'s deliberate out-of-line recursion.

#[inline]
pub(crate) fn accumulate_mana_colors(eff: &Effect, set: &mut crate::mana::ColorSet) {
    match eff {
        Effect::AddMana { pool, .. } => accumulate_payload_colors(pool, set),
        // The recursion is the only `call` site and it fires on 1 % of asks;
        // out of line the rest pay no frame. See `(-136)`.
        Effect::Seq(v) => accumulate_mana_colors_seq(v, set),
        _ => {}
    }
}

#[inline(never)]
pub(crate) fn accumulate_mana_colors_seq(v: &[Effect], set: &mut crate::mana::ColorSet) {
    v.iter().for_each(|e| accumulate_mana_colors(e, set));
}

#[inline]
pub(crate) fn accumulate_payload_colors(pool: &ManaPayload, set: &mut crate::mana::ColorSet) {
    match pool {
        ManaPayload::Colors(cs) | ManaPayload::OfColors(cs, _) => {
            cs.iter().for_each(|c| set.insert(*c))
        }
        ManaPayload::OfColor(c, _) => set.insert(*c),
        ManaPayload::AnyOneColor(_)
        | ManaPayload::AnyColorInCommanderIdentity
        | ManaPayload::AnyColors(_)
        | ManaPayload::AnyColorOpponentCouldProduce
        | ManaPayload::AnyColorYouCouldProduce
        | ManaPayload::AnyColorAGateYouControlCouldProduce
        | ManaPayload::AnyTypeTriggerSourceProduces
        | ManaPayload::AnyTypeSacrificedLandProduces
        | ManaPayload::DevotionOfChosenColor => *set = crate::mana::ColorSet::all(),
        ManaPayload::Colorless(_) => {}
        // Could produce any single color the rock was set to — treat as
        // potentially any color for the bot's mana-base reasoning.
        ManaPayload::ChosenColorOfSource
        | ManaPayload::DraftNotedColorOfSource
        | ManaPayload::ImprintedCardColor
        | ManaPayload::AnyColorAmongLegendaries
        | ManaPayload::AnyColorAmongExiledWithSource
        | ManaPayload::AnyColorAmongYourPermanents
        | ManaPayload::OneOfEachColorAmongYourPermanents => *set = crate::mana::ColorSet::all(),
        ManaPayload::Restricted(inner, _) | ManaPayload::RestrictedToChosenType(inner)
                    | ManaPayload::RestrictedToChosenTypePlain(inner)
                    | ManaPayload::RestrictedToChosenTypeOrAbility(inner)
                    | ManaPayload::RestrictedToChosenColorMono(inner) => {
            accumulate_payload_colors(inner, set)
        }
    }
}

/// A mana ability the bot is willing to count toward affordability: it
/// costs a tap and nothing the bot would regret.
///
/// We only need to know the mana *could* be paid, so color-choice sources
/// (dual lands, Birds of Paradise) and painland-style life costs count --
/// the engine's auto-tap will happily use them. Sources that consume a
/// real resource to fire (sacrifice, discard, exile, energy) are excluded:
/// counting them would have the bot commit to lines it can only pay for by
/// spending something it would rather keep.
#[inline]
pub(crate) fn is_countable_mana_ability(a: &ActivatedAbility) -> bool {
    a.tap_cost
        && a.mana_cost.symbols.is_empty()
        && !a.sac_cost
        && a.sac_other_filter.is_none()
        && a.sac_other_second.is_none()
        && a.bounce_other_filter.is_none()
        && a.tap_other_filter.is_none()
        && a.tap_n_filter.is_none()
        && a.exile_other_filter.is_none()
        && a.discard_cost.is_none()
        && !a.exile_self_cost
        && a.energy_cost == 0
        && a.collect_evidence_cost.is_none()
        && a.condition.is_none()
        && !a.from_graveyard
        && !a.from_hand
        && matches!(a.effect, Effect::AddMana { .. })
}

/// Whether a mana ability's amount is a runtime `Value` rather than a
/// constant — [`mana_ability_output`] reports one for those, which is a
/// *lower* bound and so cannot be spent as a per-colour budget.
#[inline]
pub(crate) fn mana_amount_is_dynamic(eff: &Effect) -> bool {
    use crate::effect::Value;
    let Effect::AddMana { pool, .. } = eff else { return false };
    let dynamic = |v: &Value| !matches!(v, Value::Const(_));
    match pool {
        ManaPayload::Colorless(v)
        | ManaPayload::OfColor(_, v)
        | ManaPayload::OfColors(_, v)
        | ManaPayload::AnyOneColor(v)
        | ManaPayload::AnyColors(v) => dynamic(v),
        ManaPayload::Colors(_) => false,
        // The rest are board-dependent palettes `mana_ability_output` answers
        // with a flat one — never a bound.
        _ => true,
    }
}

/// `(most mana produced, colors it could be, produces true colorless)` for
/// a mana ability's effect. Dynamic amounts (`{T}: add {G} equal to this
/// creature's power`) count as one -- enough to keep the source visible
/// without inventing a board state to measure it against.
#[inline]
pub(crate) fn mana_ability_output(eff: &Effect) -> (u32, crate::mana::ColorSet, bool) {
    use crate::effect::Value;
    use crate::mana::{Color, ColorSet};
    let mut colors = ColorSet::empty();
    accumulate_mana_colors(eff, &mut colors);
    let amount_of = |v: &Value| match v {
        Value::Const(n) => (*n).max(0) as u32,
        _ => 1,
    };
    let Effect::AddMana { pool, .. } = eff else { return (0, colors, false) };
    let (amount, colorless) = match pool {
        ManaPayload::Colors(cs) => (cs.len() as u32, false),
        ManaPayload::Colorless(v) => (amount_of(v), true),
        ManaPayload::OfColor(_, v) | ManaPayload::OfColors(_, v) => (amount_of(v), false),
        ManaPayload::AnyOneColor(v) | ManaPayload::AnyColors(v) => {
            for c in Color::ALL {
                colors.insert(c);
            }
            (amount_of(v), false)
        }
        // "Any color an opponent's land could produce" and friends: the
        // exact palette depends on a board read this estimate doesn't do,
        // so assume the source is live for any color.
        _ => {
            for c in Color::ALL {
                colors.insert(c);
            }
            (1, true)
        }
    };
    (amount, colors, colorless)
}
