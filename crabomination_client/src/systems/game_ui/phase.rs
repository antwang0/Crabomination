//! The phase chart down the left edge (its rows and the progress rail beside
//! them; the stop settings are `systems::phase_bar`'s) and the banner that
//! names each new step.

use super::*;

/// Canonical phase ordering rendered by the left-edge phase chart.
/// Kept here (not inline in `setup_game_hud`) so `update_phase_chart`
/// can reuse the same ordering / label strings.
pub(crate) const PHASE_CHART_STEPS: &[(TurnStep, &str)] = &[
    (TurnStep::Untap, "Untap"),
    (TurnStep::Upkeep, "Upkeep"),
    (TurnStep::Draw, "Draw"),
    (TurnStep::PreCombatMain, "Main 1"),
    (TurnStep::BeginCombat, "Begin Combat"),
    (TurnStep::DeclareAttackers, "Attackers"),
    (TurnStep::DeclareBlockers, "Blockers"),
    (TurnStep::CombatDamage, "Damage"),
    (TurnStep::EndCombat, "End Combat"),
    (TurnStep::PostCombatMain, "Main 2"),
    (TurnStep::End, "End"),
    (TurnStep::Cleanup, "Cleanup"),
];

/// The step as the turn line names it: the phase chart's label, or its own
/// name for first-strike damage, which the chart has no row for.
pub(crate) fn step_name(step: TurnStep) -> &'static str {
    match step {
        TurnStep::FirstStrikeDamage => "First-Strike Damage",
        step => step_short_label(step),
    }
}

pub(super) fn step_short_label(step: TurnStep) -> &'static str {
    PHASE_CHART_STEPS
        .iter()
        .find(|(s, _)| *s == step)
        .map(|(_, l)| *l)
        .unwrap_or("")
}

/// A phase chart row's segment of the progress rail down the chart's left
/// edge.
#[derive(Component)]
pub struct PhaseRail;

/// The rail's colour for a step the turn has yet to reach.
pub(super) const PHASE_RAIL_AHEAD: Color = Color::srgba(1.0, 1.0, 1.0, 0.10);

/// Where a phase chart row's step stands in the turn.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StepProgress {
    Done,
    Current,
    Ahead,
}

/// Where `row` stands when the turn is at `current`. First-strike damage
/// has no row of its own: the turn is at the damage row.
fn step_progress(row: TurnStep, current: TurnStep) -> StepProgress {
    let at = |step: TurnStep| {
        let step = if step == TurnStep::FirstStrikeDamage { TurnStep::CombatDamage } else { step };
        PHASE_CHART_STEPS.iter().position(|(s, _)| *s == step)
    };
    match (at(row), at(current)) {
        (Some(r), Some(c)) if r < c => StepProgress::Done,
        (Some(r), Some(c)) if r == c => StepProgress::Current,
        _ => StepProgress::Ahead,
    }
}

pub fn update_phase_chart(
    view: Res<CurrentView>,
    stops: Option<Res<crate::systems::phase_bar::StopConfig>>,
    ff: Option<Res<FastForward>>,
    mut rows: Query<(&PhaseStepLabel, &Children, &mut BackgroundColor), Without<PhaseRail>>,
    mut rails: Query<&mut BackgroundColor, With<PhaseRail>>,
    mut texts: Query<(&mut Text, &mut TextColor)>,
) {
    use crate::systems::phase_bar::StopMode;
    let Some(cv) = &view.0 else { return };
    let current = cv.step;
    let my_turn = cv.active_player == cv.your_seat;
    let pass_target = ff.as_ref().and_then(|f| f.pass_until);
    // The turn's progress runs down the rail in the colour of the seat whose
    // turn it is — the viewer's own on their turn, an opponent's on theirs —
    // and the current row is lit in it.
    let turn_colour = table_awareness::seat_color(cv.active_player);
    for (label, children, mut bg) in &mut rows {
        let progress = step_progress(label.0, current);
        let active = label.0 == current;
        let mode = stops
            .as_ref()
            .map(|s| s.mode(my_turn, label.0))
            .unwrap_or_default();
        // Every write in this system is compared first: it runs every frame,
        // and a rewritten `Text` is re-shaped and re-laid-out even when the
        // string is the same.
        bg.set_if_neq(BackgroundColor(match progress {
            StepProgress::Current => turn_colour.with_alpha(0.32),
            _ => Color::NONE,
        }));
        for child in children.iter() {
            if let Ok(mut rail) = rails.get_mut(child) {
                rail.set_if_neq(BackgroundColor(match progress {
                    StepProgress::Done => turn_colour.with_alpha(0.55),
                    StepProgress::Current => turn_colour.lighter(0.15),
                    StepProgress::Ahead => PHASE_RAIL_AHEAD,
                }));
            }
        }
        // Each row has exactly one Text child — rewrite its content + colour.
        // A configured stop reads as a suffix tag scoped to the kind of turn
        // currently shown (clicking the row cycles it — see
        // `phase_bar::handle_phase_chart_clicks`).
        for child in children.iter() {
            if let Ok((mut text, mut color)) = texts.get_mut(child) {
                let marker = if active {
                    "▶ "
                } else if pass_target == Some(label.0) {
                    "⏩ "
                } else {
                    "   "
                };
                let stop_tag = match mode {
                    StopMode::Auto => "",
                    StopMode::Always => "  [stop]",
                    StopMode::Skip => "  [skip]",
                };
                // CR 500.7 — flag a repeated combat/end step (extra combat,
                // Y'shtola's extra end step) on the active row so the loop reads.
                let extra_tag = if active && cv.extra_phase { "  ⟳ extra" } else { "" };
                let line = format!("{marker}{}{extra_tag}{stop_tag}", step_short_label(label.0));
                if text.0 != line {
                    text.0 = line;
                }
                color.set_if_neq(TextColor(match (active, mode) {
                    (true, _) => theme::ACCENT_YELLOW,
                    (false, StopMode::Always) => theme::ACCENT_ORANGE,
                    (false, StopMode::Skip) => theme::TEXT_MUTED.with_alpha(0.5),
                    // Steps the turn has passed recede.
                    (false, StopMode::Auto) if progress == StepProgress::Done => theme::TEXT_MUTED.with_alpha(0.6),
                    (false, StopMode::Auto) => theme::TEXT_MUTED,
                }));
            }
        }
    }
}

// ── Phase banner ──────────────────────────────────────────────────────────────

/// Remembers the last turn/step the banner system reacted to, so it only
/// flashes a banner on a genuine transition (not every view refresh).
#[derive(Resource, Default)]
pub struct PhaseBannerTracker {
    last_turn: u32,
    last_step: Option<TurnStep>,
    /// `false` until the first view is observed — priming the tracker on
    /// that first view avoids flashing a banner for whatever state the
    /// client happens to connect into (e.g. mid-mulligan).
    primed: bool,
}

/// A transient, fading center-screen banner ("Your Turn", "Combat", …).
/// `animate_phase_banner` counts `remaining` down and fades the text,
/// despawning at zero.
#[derive(Component)]
pub struct PhaseBanner {
    remaining: f32,
    total: f32,
}

const PHASE_BANNER_SECS: f32 = 1.5;

/// Peak opacity of the dark chip behind a phase banner. Faded in step with the
/// text by `animate_phase_banner`, so the chip never lingers after the text.
const PHASE_BANNER_BG_ALPHA: f32 = 0.66;

/// Flash a banner on milestone transitions: the start of a turn ("Your
/// Turn" / "<name>'s Turn") and entry into combat ("Combat"). Deliberately
/// not every step — a banner per step would be noise.
pub fn trigger_phase_banner(
    mut commands: Commands,
    view: Res<CurrentView>,
    mut tracker: ResMut<PhaseBannerTracker>,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<PhaseBanner>>,
) {
    if !view.is_changed() {
        return;
    }
    let Some(cv) = &view.0 else { return };
    if cv.game_over.is_some() {
        return;
    }

    let prev_step = tracker.last_step;
    let prev_turn = tracker.last_turn;
    let was_primed = tracker.primed;
    tracker.last_turn = cv.turn;
    tracker.last_step = Some(cv.step);
    tracker.primed = true;
    if !was_primed {
        return;
    }

    let banner: Option<(String, Color)> = if cv.turn != prev_turn {
        Some(if cv.active_player == cv.your_seat {
            ("Your Turn".to_string(), theme::ACCENT_GOLD)
        } else {
            (format!("{}'s Turn", player_name(cv, cv.active_player)), theme::TEXT_DANGER)
        })
    } else if prev_step != Some(cv.step) && cv.step == TurnStep::DeclareAttackers {
        Some(("Combat".to_string(), theme::ACCENT_ORANGE))
    } else {
        None
    };

    let Some((text, color)) = banner else { return };
    // Replace any in-flight banner so rapid transitions don't stack.
    for e in &existing {
        commands.entity(e).despawn();
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(20.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
            InGameRoot,
            PhaseBanner { remaining: PHASE_BANNER_SECS, total: PHASE_BANNER_SECS },
        ))
        .with_children(|p| {
            p.spawn((
                // Padded, semi-transparent chip behind the text so the big
                // coloured banner reads against the busy 3-D board. Its alpha
                // is faded in step with the text by `animate_phase_banner`.
                Node {
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.04, 0.07, PHASE_BANNER_BG_ALPHA)),
                Text::new(text),
                ui_fonts.tf(46.0),
                TextColor(color),
                Pickable::IGNORE,
            ));
        });
}

/// Count the banner down and fade its text (and chip background) out over the
/// last ~40% of its life; despawn when elapsed.
pub fn animate_phase_banner(
    mut commands: Commands,
    time: Res<Time>,
    mut banners: Query<(Entity, &mut PhaseBanner, &Children)>,
    mut texts: Query<&mut TextColor>,
    mut backgrounds: Query<&mut BackgroundColor>,
) {
    for (entity, mut banner, children) in &mut banners {
        banner.remaining -= crate::systems::animate::anim_dt(&time);
        if banner.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let frac = (banner.remaining / banner.total).clamp(0.0, 1.0);
        // Hold full opacity, then ease out over the final 40%.
        let alpha = (frac / 0.4).min(1.0);
        for child in children.iter() {
            if let Ok(mut tc) = texts.get_mut(child) {
                let c = tc.0.to_srgba();
                tc.0 = Color::srgba(c.red, c.green, c.blue, alpha);
            }
            // The chip background and text live on the same child entity;
            // scale its alpha off the same curve (from the fixed peak so the
            // per-frame rewrite stays idempotent).
            if let Ok(mut bg) = backgrounds.get_mut(child) {
                let c = bg.0.to_srgba();
                bg.0 = Color::srgba(c.red, c.green, c.blue, PHASE_BANNER_BG_ALPHA * alpha);
            }
        }
    }
}

#[cfg(test)]
mod phase_chart_tests {
    use super::*;

    #[test]
    fn the_rail_fills_through_the_turn() {
        use StepProgress::*;
        let at_blocks: Vec<_> = PHASE_CHART_STEPS.iter().map(|(s, _)| step_progress(*s, TurnStep::DeclareBlockers)).collect();
        assert_eq!(&at_blocks[..7], &[Done, Done, Done, Done, Done, Done, Current]);
        assert!(at_blocks[7..].iter().all(|p| *p == Ahead));
        // First-strike damage lights the damage row, the blockers done.
        assert_eq!(step_progress(TurnStep::CombatDamage, TurnStep::FirstStrikeDamage), Current);
        assert_eq!(step_progress(TurnStep::DeclareBlockers, TurnStep::FirstStrikeDamage), Done);
    }
}
