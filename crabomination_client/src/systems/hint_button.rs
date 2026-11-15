//! The Hint button (and its key, `Q` by default): ask the match what the bot
//! would do for you (`server::hint`), then show it — a "Suggested: cast
//! Lightning Bolt" banner, and a ping on the card when it is on the table.
//!
//! The suggestion is the local pilot's own pick — the same search the bots
//! play — so it takes as long as a bot's move; the banner says it is
//! thinking meanwhile. It clears when the game moves on or after
//! [`SHOW_SECS`].

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::net::ClientMsg;

use crate::net_plugin::{CurrentView, NetOutbox};
use crate::theme::{self, UiFonts};
use crate::ui_button::{Button, Interaction};

/// How long a suggestion stays up.
pub const SHOW_SECS: f32 = 10.0;

/// The hint asked for, or the one shown.
#[derive(Resource, Default)]
pub struct SuggestedMove {
    /// Asked and not yet answered.
    pub waiting: bool,
    /// The banner's text; empty when nothing is shown.
    pub text: String,
    pub card: Option<CardId>,
    /// When the answer came, for the expiry and the ping.
    pub at: f32,
    /// What the game looked like when it was asked: a suggestion for a
    /// position that has moved on is stale.
    asked_in: Option<(u32, usize, crabomination::game::TurnStep)>,
}

impl SuggestedMove {
    /// Fold the server's answer in.
    pub fn receive(&mut self, label: Option<String>, card: Option<CardId>, note: String, now: f32) {
        self.waiting = false;
        self.at = now;
        self.card = card;
        self.text = match label {
            Some(label) => format!("Suggested: {label}"),
            None => format!("No hint: {note}"),
        };
    }

    fn clear(&mut self) {
        self.waiting = false;
        self.text.clear();
        self.card = None;
    }
}

#[derive(Component)]
pub struct HintButton;

#[derive(Component)]
pub struct HintBanner;

#[derive(Component)]
pub struct HintBannerText;

/// The hint key or button: ask, once at a time.
pub fn request_hint(
    keyboard: Res<ButtonInput<KeyCode>>,
    binds: Res<crate::keybinds::Keybinds>,
    text_input: crate::systems::input_guard::TextInputGuard,
    button: Query<&Interaction, (Changed<Interaction>, With<HintButton>)>,
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut hint: ResMut<SuggestedMove>,
) {
    let key = !text_input.typing() && binds.just_pressed(&keyboard, crate::keybinds::Action::Hint);
    let clicked = button.iter().any(|i| *i == Interaction::Pressed);
    if !(key || clicked) || hint.waiting {
        return;
    }
    let (Some(outbox), Some(cv)) = (outbox, view.0.as_ref()) else { return };
    outbox.submit_msg(ClientMsg::RequestHint);
    hint.waiting = true;
    hint.text = "Thinking about your move…".to_string();
    hint.card = None;
    hint.asked_in = Some((cv.turn, cv.priority, cv.step));
}

/// Show (or clear) the banner, and ping the suggested card while it shows.
pub fn sync_hint_banner(
    mut commands: Commands,
    time: Res<Time>,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    mut hint: ResMut<SuggestedMove>,
    mut ping: ResMut<crate::systems::log_links::BoardPing>,
    banner: Query<Entity, With<HintBanner>>,
    mut text: Query<&mut Text, With<HintBannerText>>,
) {
    let now = time.elapsed_secs();
    // Stale once the position has moved on, or after a while.
    let moved_on = view.0.as_ref().zip(hint.asked_in).is_some_and(|(cv, asked)| (cv.turn, cv.priority, cv.step) != asked);
    if !hint.text.is_empty() && !hint.waiting && (moved_on || now - hint.at > SHOW_SECS) {
        hint.clear();
    }
    if hint.is_changed()
        && !hint.waiting
        && let Some(card) = hint.card
    {
        ping.ping(card, now);
    }
    match (hint.text.is_empty(), banner.iter().next()) {
        (true, Some(e)) => commands.entity(e).despawn(),
        (true, None) => {}
        (false, Some(_)) => {
            for mut t in &mut text {
                if t.0 != hint.text {
                    t.0 = hint.text.clone();
                }
            }
        }
        (false, None) => {
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        // Under the pending-cast banner's slot (64 px), which
                        // a cast the hint prompts can open.
                        top: Val::Px(104.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    HintBanner,
                    crate::systems::game_ui::InGameRoot,
                    Pickable::IGNORE,
                    GlobalZIndex(theme::layer::BANNER),
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                            border_radius: BorderRadius::all(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(theme::HUD_BG),
                        BorderColor::all(theme::ACCENT_GOLD),
                        Pickable::IGNORE,
                    ))
                    .with_children(|chip| {
                        chip.spawn((
                            Text::new(hint.text.clone()),
                            ui_fonts.tf(14.0),
                            TextColor(theme::ACCENT_GOLD),
                            HintBannerText,
                            Pickable::IGNORE,
                        ));
                    });
                });
        }
    }
}

/// The Hint button in the action column.
pub fn spawn_hint_button(p: &mut ChildSpawnerCommands, tf: &impl Fn(f32) -> TextFont, key: &str) {
    p.spawn((
        Node {
            padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
            border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
            ..default()
        },
        BackgroundColor(theme::BUTTON_NEUTRAL_BG),
        theme::HoverTint::new(theme::BUTTON_NEUTRAL_BG),
        Button,
        HintButton,
    ))
    .with_children(|b| {
        b.spawn((Text::new(format!("Hint ({key})")), tf(13.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_answer_reads_as_a_suggestion_or_a_reason() {
        let mut h = SuggestedMove { waiting: true, ..default() };
        h.receive(Some("cast Lightning Bolt".into()), Some(CardId(3)), String::new(), 1.0);
        assert!(!h.waiting);
        assert_eq!(h.text, "Suggested: cast Lightning Bolt");
        assert_eq!(h.card, Some(CardId(3)));
        h.receive(None, None, "it isn't your move".into(), 2.0);
        assert_eq!(h.text, "No hint: it isn't your move");
    }
}
