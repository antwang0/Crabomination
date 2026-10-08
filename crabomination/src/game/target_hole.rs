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
    // A hole already at slot 1 means the list is positional from there on
    // (the extra-slot walker skipped an empty run), so slot 0 holds too.
    if f0.is_some() && f1.is_some() && (f0 != f1 || additional.first().is_some_and(is_hole)) {
        *target = Some(TARGET_HOLE);
    }
}

/// CR 608.2b — "a target that's no longer in the zone it was in when it was
/// targeted is illegal": slot `slot` held a battlefield permanent as the
/// ability went on the stack (`bf_slots`) and it has left, or names a player
/// who left the game (CR 800.4a). A filter alone can't tell — a creature
/// card in a graveyard is still a creature — so an effect that finds a card
/// anywhere acted on its new object.
///
/// A declared target only — the slot has a filter: a bare `Target(n)` bound
/// from the event ("that creature" of a combat-damage trigger) isn't targeted.
pub(crate) fn target_departed(
    g: &crate::game::GameState,
    effect: &Effect,
    mode: usize,
    bf_slots: u8,
    slot: usize,
    t: &Target,
) -> bool {
    match t {
        _ if is_hole(t) => false,
        Target::Permanent(id) => {
            slot < 8
                && bf_slots & (1 << slot) != 0
                && g.battlefield_find(*id).is_none()
                && effect.target_filter_for_slot_in_mode(slot as u8, Some(mode)).is_some()
        }
        Target::Player(p) => g.players.get(*p).is_some_and(|pl| !pl.is_alive()),
    }
}

/// [`target_departed`] over a trigger's slots past the first: each departed
/// one holds its place as a hole, so its part of the effect does nothing
/// (Omo's counter reached a commander that had gone home). Returns whether
/// any of them is still legal.
pub(crate) fn hole_departed_slots(
    g: &crate::game::GameState,
    effect: &Effect,
    mode: usize,
    bf_slots: u8,
    targets: &mut [Target],
) -> bool {
    let mut any_legal = false;
    for (i, t) in targets.iter_mut().enumerate() {
        if is_hole(t) {
            continue;
        }
        if target_departed(g, effect, mode, bf_slots, i + 1, t) {
            *t = TARGET_HOLE;
        } else {
            any_legal = true;
        }
    }
    any_legal
}
