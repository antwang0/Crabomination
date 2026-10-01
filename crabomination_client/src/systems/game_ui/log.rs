//! The game log: one coloured, glyph-marked line per server event, the newest
//! on top, in a panel under the opponents.

use super::*;

/// Per-variant log color. Damage / death is red, life-gain green,
/// mana / casts gold, step changes muted, combat orange, etc. Anything
/// not specifically classified falls back to body text.
pub(super) fn event_color(ev: &crabomination::net::GameEventWire) -> Color {
    use crabomination::net::GameEventWire as E;
    match ev {
        E::DamageDealt { .. }
        | E::LifeLost { .. }
        | E::PoisonAdded { .. }
        | E::CreatureDied { .. }
        | E::PlaneswalkerDied { .. } => theme::TEXT_DANGER,

        // A seat going out of a pod is the table's biggest event short of
        // the game ending.
        E::PlayerLost { .. } | E::PlayerConceded { .. } => theme::ACCENT_ORANGE,

        // Prevented damage is a protective/beneficial outcome — colour it
        // like life-gain rather than the red damage events.
        E::LifeGained { .. } | E::DamagePrevented { .. } => theme::TEXT_GOOD,

        E::StepChanged(_) | E::TurnStarted { .. } => theme::TEXT_SECONDARY,

        // Phasing — a permanent slips out of (or back into) existence. Read
        // as a transient/secondary board event; the glyph carries the cue
        // since the permanent itself vanishes from the board view.
        E::PermanentPhasedOut { .. } => theme::TEXT_SECONDARY,

        E::CardDrawn { .. }
        | E::CardDiscarded { .. }
        | E::CardMilled { .. }
        | E::ScryPerformed { .. }
        | E::SurveilPerformed { .. }
        | E::TopCardRevealed { .. }
        | E::CardLeftGraveyard { .. } => theme::ACCENT_BLUE,

        E::ManaAdded { .. }
        | E::ColorlessManaAdded { .. }
        | E::SpellCast { .. }
        | E::AbilityActivated { .. }
        | E::LoyaltyAbilityActivated { .. }
        | E::SpellsCopied { .. } => theme::ACCENT_GOLD,

        E::CounterAdded { .. }
        | E::CounterRemoved { .. }
        | E::LoyaltyChanged { .. }
        | E::PumpApplied { .. } => theme::ACCENT_BLUE,

        E::AttackerDeclared(_)
        | E::BlockerDeclared { .. }
        | E::AttackerWentUnblocked { .. }
        | E::CombatResolved
        | E::FirstStrikeDamageResolved => theme::ACCENT_ORANGE,

        E::GameOver { .. } => theme::ACCENT_GOLD,

        // Randomization + resource gains read as gold "value" events.
        E::CoinFlipWon { .. }
        | E::CoinFlipLost { .. }
        | E::DiceRolled { .. }
        | E::Voted { .. }
        | E::VotingFinished
        | E::InitiativeTaken { .. }
        | E::EnergyGained { .. } => theme::ACCENT_GOLD,

        _ => theme::TEXT_BODY,
    }
}

/// Leading glyph for a log entry, so the red/green colour coding in
/// [`event_color`] isn't the *only* signal distinguishing harm from
/// benefit (colour-blind accessibility). `▼` = damage / life loss,
/// `✖` = a permanent died, `▲` = life gained / damage prevented. Empty
/// for everything else — the colour + text already carry those.
pub(super) fn event_glyph(ev: &crabomination::net::GameEventWire) -> &'static str {
    use crabomination::net::GameEventWire as E;
    match ev {
        E::DamageDealt { .. } | E::LifeLost { .. } | E::PoisonAdded { .. } => "▼ ",
        E::CreatureDied { .. } | E::PlaneswalkerDied { .. } => "✖ ",
        E::PlayerLost { .. } | E::PlayerConceded { .. } => "☠ ",
        E::LifeGained { .. } | E::DamagePrevented { .. } => "▲ ",
        // Coin flips and die rolls — a die glyph flags randomization outcomes.
        E::CoinFlipWon { .. } | E::CoinFlipLost { .. } | E::DiceRolled { .. } => "⚄ ",
        E::EnergyGained { .. } => "⚡ ",
        // CR 701.38 — each vote on a ballot gets a ticked-box glyph; the
        // tally that closes it gets a filled one.
        E::Voted { .. } => "☑ ",
        E::VotingFinished => "▣ ",
        // CR 726 — the initiative changing hands.
        E::InitiativeTaken { .. } => "🗝 ",
        // Phasing — a hollow circle reads as "now you don't".
        E::PermanentPhasedOut { .. } => "◌ ",
        _ => "",
    }
}

/// The event's primary card, when one is identifiable — drives the log
/// row's hover-preview. Deliberately the high-traffic variants only.
pub(super) fn event_primary_card(ev: &crabomination::net::GameEventWire) -> Option<crabomination::card::CardId> {
    use crabomination::net::GameEventWire as E;
    match ev {
        E::CardDrawn { card_id, .. }
        | E::CardDiscarded { card_id, .. }
        | E::LandPlayed { card_id, .. }
        | E::SpellCast { card_id, .. }
        | E::CardMilled { card_id, .. }
        | E::CreatureDied { card_id }
        | E::CreatureSacrificed { card_id, .. }
        | E::PermanentSacrificed { card_id, .. } => Some(*card_id),
        E::BecameTarget { target, .. } => Some(*target),
        // Ability activations are high-traffic — preview the source permanent
        // (e.g. the adapt/pump creature or the activated planeswalker).
        E::AbilityActivated { source, .. } => Some(*source),
        E::LoyaltyAbilityActivated { planeswalker, .. } => Some(*planeswalker),
        _ => None,
    }
}

/// Pretty-print a `GameEventWire` for the in-game log, resolving any
/// CardId via the running `CardNames` map (so the player sees real card
/// names instead of `CardId(N)` debug strings) and seats via the view's
/// player names (instead of `P0`/`P1`). Thin wrapper around the
/// engine-side `GameEventWire::fmt_for_log` so new event variants only
/// need a body added in one place (`crabomination/src/net.rs`).
pub(super) fn format_event(
    ev: &crabomination::net::GameEventWire,
    names: &crate::game::CardNames,
    view: Option<&crabomination::net::ClientView>,
) -> String {
    ev.fmt_for_log(
        &|id| names.get(id),
        &|seat| {
            view.map(|cv| player_name(cv, seat)).unwrap_or_else(|| format!("P{seat}"))
        },
    )
}

pub fn update_log_text(
    mut commands: Commands,
    log: Res<GameLog>,
    ui_fonts: Res<UiFonts>,
    panel_q: Query<Entity, With<GameLogPanel>>,
    mut rows: Query<(Entity, &LogRow, &mut Text, &mut TextColor)>,
) {
    if !log.is_changed() {
        return;
    }
    let Ok(panel) = panel_q.single() else { return };
    // Rows are keyed by the entry's push order and only touched where the log
    // moved: a new entry adds a row at the top, an evicted or cleared one
    // loses its row, and a coalesced repeat rewrites its own text. The log
    // used to be torn down and rebuilt — 200 text rows shaped again — on
    // every change, which during a bot's turn is several times a second.
    let entries: std::collections::HashMap<u64, &crate::game::LogEntry> =
        log.entries.iter().map(|e| (e.seq(), e)).collect();
    let mut shown = HashSet::new();
    for (row_entity, row, mut text, mut color) in &mut rows {
        match entries.get(&row.0) {
            Some(entry) => {
                shown.insert(row.0);
                if text.0 != entry.text {
                    text.0 = entry.text.clone();
                }
                color.set_if_neq(TextColor(entry.color));
            }
            None => commands.entity(row_entity).despawn(),
        }
    }
    // New entries are always newer than every shown row, so they go on top,
    // newest first.
    let mut fresh = Vec::new();
    for entry in log.entries.iter().rev().filter(|e| !shown.contains(&e.seq())) {
        // Turn dividers get a little breathing room above/below so
        // each turn reads as its own block in the scrollback.
        let node = if entry.divider {
            Node { margin: UiRect::vertical(Val::Px(3.0)), ..default() }
        } else {
            Node::default()
        };
        // Rows naming a resolvable card preview it on hover
        // (`ui_card_hover`); the rest stay click-through.
        let mut row = commands.spawn((
            Text::new(entry.text.clone()),
            ui_fonts.tf(if entry.divider { 11.0 } else { 12.0 }),
            TextColor(entry.color),
            node,
            LogRow(entry.seq()),
        ));
        match &entry.card {
            Some(card) => {
                row.insert((Button, card.clone()));
            }
            None => {
                row.insert(Pickable::IGNORE);
            }
        }
        fresh.push(row.id());
    }
    if !fresh.is_empty() {
        commands.entity(panel).insert_children(0, &fresh);
    }
}

/// A game-log row, carrying its entry's [`crate::game::LogEntry::seq`].
#[derive(Component)]
pub struct LogRow(pub u64);

/// Keep the game log positioned just below the opponent status panel. The
/// panel grows with the opponent count (one chip-row each, wrapping to a
/// couple of lines in Commander), so a fixed log `top` would overlap it for
/// 3+ players. Reads the panel's live computed height and parks the log 8px
/// under it. `ComputedNode::size` is in physical px: read as logical, the
/// log sat 45 % of the panel's height too low on a 1.45x display.
pub fn position_log_below_opponents(
    panel_q: Query<&bevy::ui::ComputedNode, With<OpponentStatusPanel>>,
    mut log_q: Query<&mut Node, With<GameLogOuterPanel>>,
) {
    let Ok(panel) = panel_q.single() else { return };
    let Ok(mut log) = log_q.single_mut() else { return };
    let panel_h = panel.size().y * panel.inverse_scale_factor();
    if panel_h <= 0.0 {
        return;
    }
    // Panel sits at top:10; place the log 8px below its bottom edge.
    let target = Val::Px(10.0 + panel_h + 8.0);
    if log.top != target {
        log.top = target;
    }
}
