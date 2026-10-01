//! Auto-advance: pass priority for the viewer through steps that need no
//! decision, per the stop settings and fast-forward.

use super::*;

/// Drives End Turn / Next Turn fast-forward: while these flags are set,
/// `auto_advance_p0` keeps submitting `PassPriority` each frame until the
/// target condition is reached.
#[derive(Resource, Default)]
pub struct FastForward {
    /// Pass until the active player is no longer us (hand off to bot's turn).
    pub end_turn: bool,
    /// Pass until we're back in our own PreCombatMain with priority.
    pub next_turn: bool,
    /// Pass until the game reaches this step (right-click a phase-chart
    /// row — roadmap "click-to-advance"). Cleared on arrival.
    pub pass_until: Option<TurnStep>,
    /// Auto-pass disabled (the "hold priority" toggle — H key / toolbar
    /// button): `auto_advance_p0` never passes for the player, every
    /// priority window is stepped through manually. Explicit fast-forwards
    /// (End Turn / Next Turn / click-to-advance) still override.
    pub manual_priority: bool,
}

// ── Auto-advance non-interactive steps ───────────────────────────────────────

pub fn auto_advance_p0(
    outbox: Option<Res<NetOutbox>>,
    view: Res<CurrentView>,
    mut ff: ResMut<FastForward>,
    stops: Option<Res<crate::systems::phase_bar::StopConfig>>,
    blocking: Res<BlockingState>,
    hold: Res<crate::systems::takeback::RewindHold>,
) {
    let Some(cv) = &view.0 else { return };
    let Some(outbox) = outbox else { return };
    if cv.game_over.is_some() { return; }
    // A take-back put the player back in a window they had acted in: leave
    // it to them, or it is passed straight away.
    if hold.holds(outbox.deliberate_count()) { return; }
    // Any pending decision suspends normal step advancement. If it's our
    // decision, the dedicated decision UI submits the answer; if it's an
    // opponent's, we just wait. Spamming `PassPriority` every frame here
    // floods the server with `DecisionPending` rejections (most visibly
    // during mulligan).
    if cv.pending_decision.is_some() { return; }
    if cv.priority != cv.your_seat { return; }
    // "Hold priority" toggle: step through every window manually. Explicit
    // fast-forwards still win (the player asked for them after toggling).
    if ff.manual_priority && !ff.end_turn && !ff.next_turn && ff.pass_until.is_none() {
        return;
    }

    let your_seat = cv.your_seat;

    // Phase-bar per-step stop override for this kind of turn (yours vs.
    // an opponent's). `Always` holds priority outright (an explicit
    // fast-forward still wins); `Skip` is folded into `should_advance`
    // below and also bypasses the combat view-holds.
    use crate::systems::phase_bar::StopMode;
    let stop_mode = stops
        .as_ref()
        .map(|s| s.mode(cv.active_player == your_seat, cv.step))
        .unwrap_or_default();
    if stop_mode == StopMode::Always && !ff.end_turn && !ff.next_turn && ff.pass_until.is_none()
    {
        return;
    }

    // Click-to-advance: arrived at the requested step — disarm and hold.
    if ff.pass_until == Some(cv.step) {
        ff.pass_until = None;
        return;
    }
    let passing_until = ff.pass_until.is_some();
    if ff.end_turn && cv.active_player != your_seat {
        ff.end_turn = false;
    }
    if ff.next_turn && cv.active_player == your_seat && cv.step == TurnStep::PreCombatMain {
        ff.next_turn = false;
        return;
    }

    // Hold priority on opp's DeclareAttackers when they declared
    // attackers — without this, the auto-pass fires before the viewer
    // sees who's swinging in, which makes the attacker lurch look like
    // it appears for the first time during DeclareBlockers. Skipped
    // when the opponent declined to attack (no `attacking`) so we
    // don't add dead-air to attack-less turns. End Turn / Next Turn
    // override the hold so the player can still skip the response
    // window when they don't care.
    if cv.step == TurnStep::DeclareAttackers
        && cv.active_player != your_seat
        && cv.battlefield.iter().any(|c| c.attacking)
        && !ff.end_turn
        && !ff.next_turn
        && !passing_until
        && stop_mode != StopMode::Skip
    {
        return;
    }

    // Don't auto-advance during interactive blocking when the viewer is
    // defending against any opponent's attack — *unless* there's nothing
    // to block: with Next Turn pressed, also skip blocker declaration when
    // either no attacker is targeting the viewer or the viewer has no
    // creature able to block (untapped, no Defender restriction —
    // summoning-sick creatures CAN block).
    // …but only until this seat's blocks are in. After that the step's
    // remaining priority windows are ordinary instant-speed windows (the
    // combat-trick slot), so fall through to the normal rules below: hold
    // when the viewer actually has a play, auto-pass when they don't —
    // rather than making them press P again on every attack.
    if cv.step == TurnStep::DeclareBlockers
        && cv.declares_blocks(your_seat)
        && !blocking.declared
    {
        use crabomination::card::Keyword;
        // We only get to DeclareBlockers if at least one attacker was
        // declared — but we still gate the *skip* on having a viable
        // blocker too. An untapped, non-Defender creature you control
        // is a candidate; summoning-sick creatures CAN block, only
        // attacking is restricted.
        let any_attacker = cv.battlefield.iter().any(|c| c.attacking);
        let any_blocker = cv.battlefield.iter().any(|c| {
            cv.may_declare_blocker(your_seat, c.owner)
                && c.is_creature()
                && !c.tapped
                && !c.keywords.contains(&Keyword::Defender)
        });
        let nothing_to_block = !any_attacker || !any_blocker;
        // An explicit per-step Skip means "never stop here" — the player
        // opted out of blocking; everything else holds the window open.
        if !((ff.next_turn || passing_until) && nothing_to_block) && stop_mode != StopMode::Skip {
            return;
        }
    }

    // Auto-pass on bookkeeping windows: non-main steps, the opponent's
    // turn (when no response is needed), and fast-forward frames.
    // On your own main phase, also auto-pass when the only things on
    // the stack are your own triggered/activated abilities (ETBs,
    // investigate, attack triggers, etc.) — those are pure bookkeeping.
    //
    // Crucially: if an opponent has a SPELL on the stack (e.g. they just
    // cast Birds of Paradise), we must NOT auto-pass — the viewer may
    // want to cast a counterspell or other response before it resolves.
    use crabomination::net::{StackItemKind, StackItemView};

    // Opponent stack items the viewer might want to respond to. Spells
    // are obvious; triggers are also responsive windows (CR 116.5 — the
    // active player passes priority first, the non-active player can
    // cast instants in response to a trigger before it resolves). Without
    // including triggers, an ETB / attack trigger on the bot's side
    // resolves before the viewer ever sees a stack pop, which the user
    // experiences as "the engine isn't pausing for priority at each
    // phase."
    let stack_has_opp_spell = cv.stack.iter().any(|item| matches!(
        item,
        StackItemView::Known(k)
            if k.controller != your_seat
                && matches!(k.kind, StackItemKind::Spell | StackItemKind::Trigger)
    ));

    let stack_is_own_triggers_only = !cv.stack.is_empty()
        && cv.stack.iter().all(|item| matches!(
            item,
            StackItemView::Known(k)
                if k.controller == your_seat && k.kind == StackItemKind::Trigger
        ));

    // End Turn (E) stops at opponent spells so the player can respond.
    // Next Turn (N) means "skip everything until my next main phase" —
    // it intentionally passes through opponent spells too.
    if stack_has_opp_spell && !ff.next_turn {
        return;
    }

    // Bookkeeping windows the viewer normally has no reason to act in.
    // (Untap has no priority window at all; the rest are pass-through steps
    // outside the main phases.)
    let bookkeeping_step = matches!(
        cv.step,
        TurnStep::Untap | TurnStep::Upkeep | TurnStep::Draw
            | TurnStep::BeginCombat | TurnStep::CombatDamage
            | TurnStep::EndCombat | TurnStep::End | TurnStep::Cleanup
    );

    // Whether the viewer actually has a legal instant-speed play in *this*
    // priority window. Every one of these lists is produced by the engine's
    // `would_accept` dry-run, so it's already priority- and timing-gated:
    // empty off-priority, and empty when the action isn't legal at the
    // current step (a sorcery in hand won't appear on the opponent's turn).
    // A non-empty list therefore means "you could legally act right now" —
    // e.g. crack a fetch land (an activatable ability) or hold up a
    // counterspell on the opponent's end step. When that's the case we stop
    // auto-passing and surface the window so the player gets priority.
    //
    // ⚠ Every list here must be checked, and for a long time only the
    // *hand*-resident ones were. A play the viewer could legally make from
    // some other zone — a card exiled by Suspend Aggression, an impulse
    // land, a plotted card, an adventure half, a Spirit Guide's pitch
    // ability in hand — left `has_instant_play` false, so this function
    // passed priority for the player and the window closed under them.
    // Reported as "sometimes I don't get the chance to play instants before
    // damage is applied": the "sometimes" was whether the only playable
    // thing happened to be in hand.
    let has_instant_play = !cv.castable_hand.is_empty()
        || !cv.back_castable_hand.is_empty()
        || !cv.prototypable_hand.is_empty()
        || !cv.prepare_castable.is_empty()
        || !cv.spliceable_hand.is_empty()
        || !cv.activatable_permanents.is_empty()
        || !cv.activatable_loyalty.is_empty()
        || !cv.kickable_hand.is_empty()
        || !cv.kicker_option_sets.is_empty()
        || !cv.buyback_hand.is_empty()
        // Non-hand zones, same `would_accept` validation as the rest.
        || !cv.may_play_castable.is_empty()
        || !cv.may_play_lands.is_empty()
        || !cv.castable_plotted.is_empty()
        || !cv.adventure_exile.is_empty()
        || !cv.hand_activatable.is_empty()
        // CR 903.8 — a flash commander castable from the command zone.
        || !cv.castable_command.is_empty();

    // Auto-pass a window only when the viewer has nothing to do there —
    // unless they've explicitly asked to fast-forward. End Turn (E) skips
    // the rest of the viewer's own turn; Next Turn (N) skips the whole turn
    // cycle up to the viewer's next main phase. The stack-of-own-triggers
    // case is pure bookkeeping (ETBs, investigate, attack triggers) and
    // always advances so those don't strand the player.
    let should_advance = ff.end_turn
        || ff.next_turn
        || passing_until
        || cv.step == TurnStep::Untap
        || stack_is_own_triggers_only
        || stop_mode == StopMode::Skip
        || ((bookkeeping_step || cv.active_player != your_seat) && !has_instant_play);

    if should_advance {
        outbox.submit_auto(GameAction::PassPriority);
    }
}
