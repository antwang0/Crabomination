//! CR 601.2c / 700.2 — the target slots of a multi-mode cast
//! (`GameAction::CastSpellSpree` on a `ChooseModesCast` /
//! `ChooseModesByPoints` spell). Each chosen target-bearing mode takes the
//! next slot, in printed order, which is how resolution binds them; the
//! mode-agnostic slot lookup instead read slot N as "the first mode that
//! surfaces a slot N", so every mode's own slot 0 collapsed onto mode 0's
//! filter. Dromoka's Command's "target player sacrifices an enchantment" +
//! "+1/+1 counter on target creature" was refused because the player was
//! checked against mode 0's "instant or sorcery spell".

use crate::card::SelectionRequirement;
use crate::effect::Effect;

/// The cast-time filter for `slot` of a multi-mode cast choosing `chosen`
/// (sorted, repeats kept), or `None` when `effect` isn't a cast-time modal
/// spell or no modes were chosen — the caller then keeps its ordinary lookup.
/// `Some(None)` is a slot no chosen mode targets through.
pub(crate) fn chosen_mode_slot_filter<'a>(
    effect: &'a Effect,
    chosen: &[u8],
    slot: u8,
    kicked: bool,
) -> Option<Option<&'a SelectionRequirement>> {
    let (Effect::ChooseModesCast { modes, .. } | Effect::ChooseModesByPoints { modes, .. }) = effect else {
        return None;
    };
    if chosen.is_empty() {
        return None;
    }
    // A mode owns as many consecutive slots as it declares (a fight mode's
    // two creatures); find the one covering `slot` and its own slot index.
    let mut first = 0usize;
    for m in chosen.iter().filter_map(|&i| modes.get(i as usize)).filter(|m| m.requires_target()) {
        let k = crate::game::effects::mode_slot_count(m);
        if (slot as usize) < first + k {
            return Some(m.target_filter_for_slot_in_mode_kicked((slot as usize - first) as u8, None, kicked));
        }
        first += k;
    }
    Some(None)
}
