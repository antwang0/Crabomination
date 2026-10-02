//! CR 601.2c + 601.2 — a spell announces a target for each target it requires,
//! and a cast that can't complete a step is illegal. A cast arriving with no
//! target at all is rejected when slot 0 is required: nothing aims an empty
//! slot later, so such a spell used to resolve for nobody.

use super::GameState;
use crate::card::CardInstance;
use crate::effect::Effect;

impl GameState {
    /// Whether an instant or sorcery cast with **no** targets requires a slot-0
    /// target. Slots the card marks optional ("up to one") never count, nor
    /// do stack-object filters (the board walk can't see spells or
    /// abilities) or a filter only a kicker or mode it wasn't cast with asks
    /// for.
    pub(crate) fn required_target_missing(
        &self,
        card: &CardInstance,
        effect: &Effect,
        mode: Option<usize>,
        kicked: bool,
        x: u32,
    ) -> bool {
        let def = &card.definition;
        (def.is_instant() || def.is_sorcery()) && slot0_required(effect, mode, kicked, x)
    }
}

/// Whether `effect` requires a slot-0 target (CR 601.2c, and CR 602.2b for
/// an activated ability): declared, not optional, not a stack-object filter,
/// and not behind a mode chosen later.
pub(crate) fn slot0_required(effect: &Effect, mode: Option<usize>, kicked: bool, x: u32) -> bool {
    let Some(filter) = effect.target_filter_for_slot_in_mode_kicked(0, mode, kicked) else {
        return false;
    };
    if filter.mentions_stack_object()
        || effect.target_slot_optional_x(0, mode, x)
        || effect.slot_owner(0, mode).is_some_and(deferred)
        || under_may(effect)
    {
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
    !has_mode_choice(view)
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
            | Effect::ChooseSomeAmong { .. }
            | Effect::ChooseOneAtRandomAmong { .. }
    ) {
        return true;
    }
    let mut found = false;
    e.for_each_inner(&mut |inner| found |= has_mode_choice(inner));
    found
}

/// A body that resolves later — a delayed trigger or a replacement of your
/// next draw — whose "target" is chosen when it fires or applies, not as the
/// spell is cast or the ability activated (Words of War, Ride the Avalanche).
pub(crate) fn deferred(e: &Effect) -> bool {
    matches!(
        e,
        Effect::ReplaceYourNextDrawThisTurn { .. }
            | Effect::DelayUntil { .. }
            | Effect::DelayUntilWithCapture { .. }
            | Effect::OnAttackedUntilYourNextTurn { .. }
            | Effect::OnMatchingAttacksThisTurn { .. }
            | Effect::OnMatchingBlocksThisTurn { .. }
            | Effect::CreaturesYouControlEnteringThisTurn { .. }
            | Effect::CreaturesYouControlDyingThisTurn { .. }
            | Effect::WheneverCreatureDiesThisTurn { .. }
            | Effect::WheneverCreatureEntersThisTurn { .. }
            | Effect::WheneverCreatureEntersUntilYourNextTurn { .. }
            | Effect::CreaturesYouControlDealingCombatDamageThisTurn { .. }
            | Effect::WheneverYouGainLifeThisTurn { .. }
            | Effect::WheneverCombatDamageToYouPreventedThisTurn { .. }
            | Effect::WheneverOpponentMakesYouDiscardThisTurn { .. }
            | Effect::WheneverCardEntersOpponentGraveyardThisTurn { .. }
            | Effect::OnEachSpellCastThisTurn { .. }
            | Effect::OnEachSpellYouCastUntilEndOfYourNextTurn { .. }
            | Effect::OnYourNextSpellCastThisTurn { .. }
            | Effect::OnYourNextExhaustActivationThisTurn { .. }
            | Effect::OnYourNextAttackThisTurn { .. }
            | Effect::OnYourNextInstantSorceryThisTurn { .. }
            | Effect::OnYourNextSpellOfTypeThisTurn { .. }
            | Effect::OnYourNextSpellMatchingThisTurn { .. }
            | Effect::OnYourNextNamedSpellThisTurn { .. }
            | Effect::AtEachCombatThisTurn { .. }
            | Effect::WhenLastCreatedTokenLeaves { .. }
    )
}

/// Slot 0 is declared inside a "you may" wrapper — the catalog's shape for
/// "up to one target" on an ability (Geyadrone Dihada's +1), so the slot may
/// be left empty.
fn under_may(e: &Effect) -> bool {
    let declares = |x: &Effect| x.target_filter_for_slot_in_mode_kicked(0, None, false).is_some();
    match e {
        Effect::Seq(v) => v.iter().find(|x| declares(x)).is_some_and(under_may),
        Effect::MayDo { body, .. }
        | Effect::MayDoBy { body, .. }
        | Effect::MayPay { body, .. }
        | Effect::MayPayBy { body, .. }
        | Effect::MayPayLife { body, .. }
        | Effect::MaySacrifice { then: body, .. }
        | Effect::MayTap { then: body, .. }
        | Effect::MayDiscard { then: body, .. } => declares(body),
        _ => false,
    }
}
