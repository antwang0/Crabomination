//! CR 115.1 / 601.2c — an optional target slot left empty ahead of a filled
//! one. The target list is positional (`Target(n)` reads slot `n`), so a
//! trigger whose "up to one target …" slot 0 found nothing while slot 1 found
//! something used to bind slot 1's pick as slot 0 — Betor's graveyard
//! creature became the "+1/+1 counters on up to one other target creature".
//! The hole holds slot 0's place: card ids start at 1 (`next_id`), so
//! `CardId(0)` names no object, every selector reading it finds nothing, and
//! the slot's effect does nothing.

use crate::card::CardId;
use crate::effect::Effect;
use crate::game::types::Target;

/// The placeholder for an optional slot nobody filled.
pub const TARGET_HOLE: Target = Target::Permanent(CardId(0));

pub fn is_hole(t: &Target) -> bool {
    *t == TARGET_HOLE
}

/// Slot 0 of an auto-targeted push stays empty while later slots were
/// filled: hold its place when slot 0 is optional and its filter differs
/// from slot 1's. Same-filter slots ("up to two target creatures") are
/// interchangeable, so those keep shifting down as they always have.
pub fn pad_empty_slot_zero(effect: &Effect, mode: Option<usize>, target: &mut Option<Target>, additional: &[Target]) {
    if target.is_some() || additional.is_empty() || !effect.target_slot_optional(0, mode) {
        return;
    }
    let f0 = effect.target_filter_for_slot_in_mode_kicked(0, mode, false);
    let f1 = effect.target_filter_for_slot_in_mode_kicked(1, mode, false);
    if f0.is_some() && f1.is_some() && f0 != f1 {
        *target = Some(TARGET_HOLE);
    }
}
