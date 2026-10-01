//! The prompt line under the action buttons: what the viewer can do right now.

use super::*;

pub fn update_hint(
    view: Res<CurrentView>,
    targeting: Res<TargetingState>,
    legal_targets: Res<crate::game::LegalTargets>,
    blocking: Res<BlockingState>,
    time: Res<Time>,
    mut q: Query<(&mut Text, &mut TextColor, &mut TextFont), With<HintText>>,
) {
    let Ok((mut t, mut color, mut font)) = q.single_mut() else { return };
    let Some(cv) = &view.0 else { return };
    if cv.game_over.is_some() {
        apply_hint(&mut t, &mut color, &mut font, String::new(), theme::ACCENT_GOLD, 13.0);
        return;
    }
    if targeting.active {
        let (msg, hint_color, hint_size) = if targeting.pending_decision_target {
            let src = if legal_targets.source_name.is_empty() {
                "A triggered ability".to_string()
            } else {
                legal_targets.source_name.clone()
            };
            let body = if legal_targets.description.is_empty() {
                "needs a target".to_string()
            } else {
                legal_targets.description.clone()
            };
            let skip = if legal_targets.declinable { " Esc skips it." } else { "" };
            (
                format!("⚡ {src}: {body}. Click / Enter on a highlighted target.{skip}"),
                theme::ACCENT_BLUE,
                15.0_f32,
            )
        } else if legal_targets.description.is_empty() {
            (
                "Click / Enter on a target. Tab,← → select. Esc cancels.".to_string(),
                theme::ACCENT_GOLD,
                13.0_f32,
            )
        } else {
            // Cast-time slot 0, named: "Together as One: draw cards" tells the
            // player which half of a multi-target spell they are aiming.
            (
                format!(
                    "🎯 {}: {}. Click / Enter on a target. Tab,← → select. Esc cancels.",
                    legal_targets.source_name, legal_targets.description,
                ),
                theme::ACCENT_GOLD,
                13.0_f32,
            )
        };
        apply_hint(&mut t, &mut color, &mut font, msg, hint_color, hint_size);
        return;
    }
    if !cv.stack.is_empty() {
        let your_priority = cv.priority == cv.your_seat;
        use crabomination::net::StackItemKind;
        // Style the hint based on the top-of-stack kind so triggered
        // abilities visually stand out from spells (and from idle
        // status messages). Triggers are the most "surprising" event —
        // they can fire without the player initiating anything — so
        // they get the boldest treatment.
        let (style_text, style_color, style_size) = if your_priority {
            match cv.stack.last() {
                Some(StackItemView::Known(k)) => {
                    let ctrl = if k.controller == cv.your_seat {
                        "Your".to_string()
                    } else {
                        format!("{}'s", player_name(cv, k.controller))
                    };
                    let tgt = match &k.target {
                        Some(Target::Player(s)) => format!(
                            " targeting {}",
                            if *s == cv.your_seat { "you".into() } else { player_name(cv, *s) }
                        ),
                        Some(Target::Permanent(id)) => cv
                            .battlefield
                            .iter()
                            .find(|p| p.id == *id)
                            .map(|p| format!(" targeting {}", p.name))
                            .unwrap_or_default(),
                        None => String::new(),
                    };
                    match k.kind {
                        StackItemKind::Trigger => (
                            format!(
                                "⚡ TRIGGER: {} {}{}. Respond from hand, or Space to let it resolve.",
                                ctrl, k.name, tgt
                            ),
                            theme::ACCENT_BLUE,
                            16.0_f32,
                        ),
                        StackItemKind::Spell => (
                            format!(
                                "↻ {} {} on stack{}. Respond from hand, or Space to let it resolve.",
                                ctrl, k.name, tgt
                            ),
                            theme::ACCENT_ORANGE,
                            14.0_f32,
                        ),
                    }
                }
                _ => (
                    format!("{} item(s) on stack. Space = let it resolve.", cv.stack.len()),
                    theme::ACCENT_GOLD,
                    13.0_f32,
                ),
            }
        } else {
            (
                format!("Waiting for {} to act on the stack.", player_name(cv, cv.priority)),
                theme::TEXT_SECONDARY,
                13.0_f32,
            )
        };
        apply_hint(&mut t, &mut color, &mut font, style_text, style_color, style_size);
        return;
    }
    let your_seat = cv.your_seat;
    let viewer_is_defending = cv.step == TurnStep::DeclareBlockers
        && cv.declares_blocks(your_seat)
        && cv.priority == your_seat
        && !blocking.declared;
    if viewer_is_defending {
        let msg = if blocking.selected_blocker.is_some() {
            "Click / Enter on an attacker to assign the block. Esc cancels.".to_string()
        } else {
            "Click / Enter on a creature to block with it. Tab,← → select. P skip.".to_string()
        };
        apply_hint(&mut t, &mut color, &mut font, msg, theme::ACCENT_GOLD, 13.0);
        return;
    }
    // Master Warcraft — someone took over this turn's declarations. Say who,
    // and (when it's the viewer) whose creatures they're picking from.
    if let Some(chooser) = cv.combat_chooser
        && matches!(cv.step, TurnStep::DeclareAttackers | TurnStep::DeclareBlockers)
    {
        let attacks = cv.step == TurnStep::DeclareAttackers;
        let what = if attacks { "attackers" } else { "blocks" };
        let msg = if chooser == your_seat {
            let whose = if attacks {
                cv.active_player
            } else {
                default_attack_target_seat(cv)
            };
            if whose == your_seat {
                format!("You choose this turn's {what}. P = declare none.")
            } else {
                format!(
                    "You choose this turn's {what} — from {}'s creatures. P = declare none.",
                    player_name(cv, whose)
                )
            }
        } else {
            format!("{} chooses this turn's {what}.", player_name(cv, chooser))
        };
        apply_hint(&mut t, &mut color, &mut font, msg, theme::ACCENT_GOLD, 13.0);
        return;
    }
    // Whose *move* it is is priority, not whose turn it is. Keying this on
    // the turn player told a viewer holding priority on the opponent's turn
    // — the whole window in which you respond to their spell — that the
    // opponent was "thinking", while the game sat waiting on the viewer.
    let you_have_priority = cv.priority == your_seat;
    let body = match (you_have_priority, cv.active_player == your_seat, cv.step) {
        (true, true, TurnStep::PreCombatMain) | (true, true, TurnStep::PostCombatMain) => {
            "Click / Enter to play. Tab,← →: select. F flip · L alt · M ability. P pass.".to_string()
        }
        (true, true, TurnStep::DeclareAttackers) => {
            "A = Attack with all eligible creatures. P = Pass (no attack).".to_string()
        }
        (true, true, TurnStep::DeclareBlockers) => {
            "Opponent is assigning blocks. P = Proceed to combat.".to_string()
        }
        (true, true, _) => "You have priority. P to pass.".to_string(),
        // The opponent's turn, but the viewer holds priority: this is the
        // response window, and it is waiting on them.
        (true, false, _) => {
            "Your priority — respond, or P to pass.".to_string()
        }
        (false, ..) => {
            // Cycle 1→2→3 dots every ~0.4s so the player can tell the
            // bot is still working and the game hasn't hung. Stick on a
            // visible dot count (never zero) so the line is the same
            // width every frame. Named off `priority`, so it only ever
            // shows while that seat really is the one to act.
            let phase = (time.elapsed_secs() / 0.4) as u64 % 3;
            let dots = match phase {
                0 => ".  ",
                1 => ".. ",
                _ => "...",
            };
            format!("{} is thinking{}", player_name(cv, cv.priority), dots)
        }
    };
    apply_hint(&mut t, &mut color, &mut font, body, theme::ACCENT_GOLD, 13.0);
}

/// Apply `(text, colour, size)` to the hint chip, skipping the write
/// when the field already matches. Avoids spurious change-detection
/// fanout from per-frame writes (this system runs every frame).
///
/// Takes the `Mut`s themselves: coercing a `Mut<Text>` to `&mut Text` is
/// a `DerefMut`, which marks the component changed before any comparison
/// here could skip the write.
fn apply_hint(
    text: &mut Mut<Text>,
    color: &mut Mut<TextColor>,
    font: &mut Mut<TextFont>,
    new_text: String,
    new_color: Color,
    new_size: f32,
) {
    if text.0 != new_text {
        text.0 = new_text;
    }
    if color.0 != new_color {
        color.0 = new_color;
    }
    // `TextFont::font_size` is a `FontSize` enum in Bevy 0.19; the UI sets
    // sizes in logical pixels, so compare/normalize on the `Px` value.
    let cur_px = match font.font_size {
        FontSize::Px(v) => v,
        _ => f32::INFINITY,
    };
    if (cur_px - new_size).abs() > f32::EPSILON {
        font.font_size = FontSize::Px(new_size);
    }
}

/// Sync the dark-tinted hint chip's visibility to the current
/// `HintText` content. Lives in its own system so every early-return
/// path in `update_hint` (game-over, targeting, stack, blocking, etc.)
/// is covered without per-branch ceremony.
pub fn sync_hint_chip_visibility(
    text_q: Query<&Text, With<HintText>>,
    mut chip_q: Query<&mut Node, With<HintChip>>,
) {
    let Ok(text) = text_q.single() else { return };
    let Ok(mut chip) = chip_q.single_mut() else { return };
    let target = if text.0.is_empty() { Display::None } else { Display::Flex };
    if chip.display != target {
        chip.display = target;
    }
}
