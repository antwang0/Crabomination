//! CR 601.2c + 601.2 — a spell announces a target for each target it requires,
//! and a cast that can't complete a step is illegal. A cast arriving with no
//! target at all is only rejected when nothing could fill slot 0: the engine
//! auto-aims an empty slot at resolution, so a legal candidate keeps the old
//! path.

use super::GameState;
use crate::card::CardInstance;
use crate::effect::Effect;

impl GameState {
    /// Whether an instant or sorcery cast with **no** targets requires a slot-0
    /// target that no object or player could fill. Slots the card marks
    /// optional ("up to one") never count, nor do stack-object filters (the
    /// board walk can't see spells or abilities) or a filter only a kicker or
    /// mode it wasn't cast with asks for.
    pub(crate) fn required_target_unfillable(
        &self,
        caster: usize,
        card: &CardInstance,
        effect: &Effect,
        mode: Option<usize>,
        kicked: bool,
        x: u32,
    ) -> bool {
        let def = &card.definition;
        if !(def.is_instant() || def.is_sorcery()) {
            return false;
        }
        let Some(filter) = effect.target_filter_for_slot_in_mode_kicked(0, mode, kicked) else {
            return false;
        };
        if filter.mentions_stack_object() || effect.target_slot_optional_x(0, mode, x) {
            return false;
        }
        // A mode picked here narrows to that mode; any other mode choice
        // (unpicked, "choose N", points) settles later, so the gate stays out.
        let view = match (effect, mode) {
            (Effect::ChooseMode(modes), Some(m)) => match modes.get(m) {
                Some(v) => v,
                None => return false,
            },
            _ => effect,
        };
        if has_mode_choice(view) {
            return false;
        }
        self.enumerate_legal_targets_xc(view, caster, Some(card.id), x, 0).is_empty()
            && self
                .auto_target_for_effect_avoiding_set_xc(view, caster, &[card.id], x, 0)
                .is_none()
    }
}

fn has_mode_choice(e: &Effect) -> bool {
    if matches!(
        e,
        Effect::ChooseMode(_)
            | Effect::ChooseN { .. }
            | Effect::ChooseUpToN { .. }
            | Effect::ChooseModesCast { .. }
            | Effect::ChooseModesByPoints { .. }
            | Effect::ChooseUnchosenMode { .. }
            | Effect::ChooseUnchosenModeThisTurn { .. }
            | Effect::ChooseOneAmong { .. }
            | Effect::ChooseOneAtRandomAmong { .. }
    ) {
        return true;
    }
    let mut found = false;
    e.for_each_inner(&mut |inner| found |= has_mode_choice(inner));
    found
}
