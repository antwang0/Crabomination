//! Shape questions about a printed mana ability: can an estimate count it,
//! how much and which colours it makes, is its amount a runtime value.
//!
//! Shared by `server::bot::available_mana` and
//! `game::actions::mana_summary_of` (PERF `(-317)`); here because `actions.rs`
//! must not depend on `server::bot`, and one copy beats two hand-written ones.
//!
//! Every function is pure in the ability/effect (no board, instance or seat),
//! which is what makes `mana_summary_of`'s per-definition pack sound.

use crate::effect::{ActivatedAbility, Effect, ManaPayload};

// ⚠ `#[inline]` on each of these is load-bearing: `release-fast` has 16
// codegen units and no LTO, so without it a cross-module callee is a call
// (moving the file without it cost +0.10..0.18 %). `accumulate_mana_colors_seq`
// keeps `#[inline(never)]` — PERF `(-136)`'s deliberate out-of-line recursion.

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
        | ManaPayload::CircledColors
        | ManaPayload::AnyColors(_)
        | ManaPayload::DifferentColors(_)
        | ManaPayload::AnyColorOpponentCouldProduce
        | ManaPayload::AnyColorYouCouldProduce
        | ManaPayload::AnyColorABasicLandYouControlCouldProduce
        | ManaPayload::AnyColorAGateYouControlCouldProduce
        | ManaPayload::AnyTypeALandYouControlCouldProduce(_)
        | ManaPayload::AnyTypeAGateYouControlCouldProduce
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
        | ManaPayload::AnyColorAmongLegendaryCreatureCardsInYourGraveyard
        | ManaPayload::AnyColorAmongCardsInYourGraveyard
        | ManaPayload::AnyColorOfSource
        | ManaPayload::AnyColorAmongExiledWithSource
        | ManaPayload::AnyColorAmongYourPermanents
        | ManaPayload::OneOfEachColorAmongYourPermanents
        | ManaPayload::OneOfEachColorAmongYourMonocoloredPermanents
        | ManaPayload::OneOfEachColorAmongExiledWithSource => *set = crate::mana::ColorSet::all(),
        ManaPayload::Restricted(inner, _) | ManaPayload::RestrictedToChosenType(inner)
                    | ManaPayload::RestrictedToChosenTypePlain(inner)
                    | ManaPayload::RestrictedToChosenTypeOrAbility(inner)
                    | ManaPayload::RestrictedToChosenTypeSpellOrAbility(inner)
                    | ManaPayload::RestrictedToChosenTypeOrLegendary(inner)
                    | ManaPayload::RestrictedToChosenColorMono(inner) => {
            accumulate_payload_colors(inner, set)
        }
    }
}

/// A mana ability the bot is willing to count toward affordability: it
/// costs a tap and nothing the bot would regret.
///
/// Colour-choice sources and painland life costs count (auto-tap uses them);
/// sources that consume a real resource (sacrifice, discard, exile, energy)
/// don't, so the bot never commits to lines that spend what it would keep.
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

/// A mana ability whose X is a non-mana cost the activator sizes — "{T},
/// Sacrifice X Goats: Add X mana" (Springjack Pasture), "{T}, Tap X untapped
/// tokens you control: Add X mana" (Hazel). Activated with no X it makes
/// nothing, so the auto-tapper sizes it to the shortfall instead.
#[inline]
pub(crate) fn is_x_sized_mana_ability(a: &ActivatedAbility) -> bool {
    (a.sac_other_x || a.tap_n_x) && !a.mana_cost.has_x()
}

/// Whether a mana ability makes mana only *some* payments may spend —
/// Gideon's Memorial's "spend this mana only to cast planeswalker spells",
/// Cavern of Souls' chosen-type mana. A rider (`SpendRestriction::is_rider`)
/// permits every payment, so it is not one.
///
/// Kept out of the auto-tapper's source table (the kind-aware
/// `pay_with_restricted_sources` retry taps them when they may fund the
/// payment); listed, one was tapped for a pip it could not pay (Gideon's
/// Memorial, 2026-11-22).
pub(crate) fn mana_is_spend_restricted(eff: &Effect) -> bool {
    let Effect::AddMana { pool, .. } = eff else { return false };
    match pool {
        ManaPayload::Restricted(_, r) => !r.is_rider(),
        ManaPayload::RestrictedToChosenType(_)
        | ManaPayload::RestrictedToChosenTypePlain(_)
        | ManaPayload::RestrictedToChosenTypeOrAbility(_)
        | ManaPayload::RestrictedToChosenTypeSpellOrAbility(_)
        | ManaPayload::RestrictedToChosenTypeOrLegendary(_)
        | ManaPayload::RestrictedToChosenColorMono(_) => true,
        _ => false,
    }
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
        | ManaPayload::AnyColors(v)
        | ManaPayload::DifferentColors(v) => dynamic(v),
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
        ManaPayload::AnyOneColor(v) | ManaPayload::AnyColors(v) | ManaPayload::DifferentColors(v) => {
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
