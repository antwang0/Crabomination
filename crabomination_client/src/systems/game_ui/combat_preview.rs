//! The combat preview panel: what the declared attacks and blocks do, before
//! damage.

use super::*;

// ── Combat preview ────────────────────────────────────────────────────────────

/// Rebuild the top-center combat-preview panel from `combat_preview`,
/// showing each player's projected life swing for the *currently declared*
/// attackers/blocks: `<who>: <life> → <new>` (red when dropping, green when
/// gaining), with a `(+N lifelink)` note where lifelink applies. Hidden
/// outside combat or when no player's life actually changes. Surfaces the
/// damage AND lifelink figures the engine already projects — the "who
/// dies" half is the red dying-creature borders (`update_dying_highlights`).
pub fn update_combat_preview_panel(
    view: Res<CurrentView>,
    mut commands: Commands,
    mut panel_q: Query<(Entity, &mut Node), With<CombatPreviewPanel>>,
    ui_fonts: Res<UiFonts>,
) {
    if !view.is_changed() {
        return;
    }
    let Ok((panel, mut node)) = panel_q.single_mut() else { return };
    commands.entity(panel).despawn_children();

    let hide = |node: &mut Node| node.display = Display::None;
    let Some(cv) = &view.0 else { hide(&mut node); return };
    let Some(cp) = &cv.combat_preview else { hide(&mut node); return };

    let lookup = |table: &[(usize, i32)], seat: usize| {
        table.iter().find(|(s, _)| *s == seat).map(|(_, v)| *v).unwrap_or(0)
    };

    // One row per player whose life actually changes, ordered by seat.
    let mut seats: Vec<usize> = cv.players.iter().map(|p| p.seat).collect();
    seats.sort_unstable();
    let mut rows: Vec<(String, Color)> = Vec::new();
    for seat in seats {
        let Some(p) = cv.players.iter().find(|p| p.seat == seat) else { continue };
        let dmg = lookup(&cp.damage_to_players, seat);
        let gain = lookup(&cp.lifegain_to_players, seat);
        if dmg == 0 && gain == 0 {
            continue;
        }
        let new_life = p.life - dmg + gain;
        let who = if seat == cv.your_seat { "You".to_string() } else { player_name(cv, seat) };
        let note = if gain > 0 { format!("  (+{gain} lifelink)") } else { String::new() };
        // Flag a projected death (CR 104.3a — life ≤ 0 loses) so the player
        // sees a lethal swing at a glance instead of decoding a negative total.
        let lethal = new_life <= 0;
        let color = if lethal || new_life < p.life {
            theme::TEXT_DANGER
        } else if new_life > p.life {
            theme::TEXT_GOOD
        } else {
            theme::TEXT_BODY
        };
        let lethal_tag = if lethal { "  ☠ LETHAL" } else { "" };
        rows.push((format!("{who}:  {} → {}{}{}", p.life, new_life, note, lethal_tag), color));
    }

    // One row per attacked planeswalker projected to lose loyalty.
    for (pw, dmg) in &cp.damage_to_planeswalkers {
        let Some(p) = cv.battlefield.iter().find(|c| c.id == *pw) else { continue };
        let loyalty = p
            .counters
            .iter()
            .find(|(k, _)| matches!(k, crabomination::card::CounterType::Loyalty))
            .map(|(_, n)| *n as i32)
            .unwrap_or(0);
        let new_loyalty = (loyalty - dmg).max(0);
        let color = if p.controller == cv.your_seat { theme::TEXT_DANGER } else { theme::TEXT_GOOD };
        rows.push((format!("{}:  {} → {}", p.name, loyalty, new_loyalty), color));
    }

    // Numeric "who dies" summary to complement the red dying-creature borders:
    // split the projected casualties by controller so the player sees the
    // trade at a glance (yours in danger-red, opponents' in good-green).
    let (mut yours, mut theirs) = (0u32, 0u32);
    for id in &cp.dying_creatures {
        match cv.battlefield.iter().find(|c| c.id == *id) {
            Some(c) if c.controller == cv.your_seat => yours += 1,
            Some(_) => theirs += 1,
            None => {}
        }
    }
    if yours > 0 || theirs > 0 {
        let mut parts = Vec::new();
        if theirs > 0 { parts.push(format!("{theirs} theirs")); }
        if yours > 0 { parts.push(format!("{yours} yours")); }
        let color = if yours > theirs { theme::TEXT_DANGER } else { theme::TEXT_GOOD };
        rows.push((format!("Dying:  {}", parts.join(", ")), color));
    }

    if rows.is_empty() {
        hide(&mut node);
        return;
    }
    node.display = Display::Flex;
    let tf = |s: f32| ui_fonts.tf(s);
    commands.entity(panel).with_children(|p| {
        p.spawn((Text::new("Combat"), tf(11.0), TextColor(theme::ACCENT_ORANGE), Pickable::IGNORE));
        for (text, color) in rows {
            p.spawn((Text::new(text), tf(14.0), TextColor(color), Pickable::IGNORE));
        }
    });
}
