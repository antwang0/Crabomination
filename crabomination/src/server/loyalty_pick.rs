//! A "+1: Add {R}{R}" loyalty ability always outscored Chandra, Torch of
//! Defiance's "+1: exile the top card, you may cast it; if you don't, 2 damage
//! to each opponent": the outcome eval prices floating mana and not an exiled
//! card the bot may cast, so the impulse was never activated in 183 pod decks.

use crate::effect::{Effect, LoyaltyAbility};

/// True when `ability` only adds mana.
fn adds_mana_only(ability: &LoyaltyAbility) -> bool {
    matches!(ability.effect, Effect::AddMana { .. })
}

/// True when `ability` exiles the top of its controller's library with a
/// permission to cast it (an impulse draw).
fn impulse(ability: &LoyaltyAbility) -> bool {
    matches!(ability.effect, Effect::ExileTopAndGrantMayPlay { .. })
}

/// Whether a mana-only loyalty ability (the ability at index `idx`) should be
/// dropped from the candidates because an impulse ability of the same walker
/// is also a candidate — the impulse card is castable off lands, and an uncast
/// one still pays out.
pub(super) fn drop_mana_for_impulse(effective: &[LoyaltyAbility], candidates: &[usize], idx: usize) -> bool {
    effective.get(idx).is_some_and(adds_mana_only)
        && candidates.iter().any(|&i| effective.get(i).is_some_and(impulse))
}

/// `loyalty_fixes`: whether a mana-only loyalty ability (index `idx`) should
/// be dropped because the same walker has a non-mana candidate costing no
/// more loyalty. The mana empties at the end of the phase unless a spell
/// spends it, and the outcome eval scored the floating mana above a surveil:
/// Chandra, Chill of Compliance took "+1: Add {U}" 374 times against 19
/// "+1: Surveil 1" (2026-11-22).
pub(super) fn drop_mana_for_plus(effective: &[LoyaltyAbility], candidates: &[usize], idx: usize) -> bool {
    let Some(mana) = effective.get(idx).filter(|a| adds_mana_only(a)) else { return false };
    candidates.iter().any(|&i| {
        i != idx && effective.get(i).is_some_and(|a| !adds_mana_only(a) && a.loyalty_cost >= mana.loyalty_cost)
    })
}
