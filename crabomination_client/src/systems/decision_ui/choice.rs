//! One button that answers a decision. Most decisions are answered with a
//! single click — a colour, a mode, a ballot option, which legend to keep, a
//! card name, Learn's three kinds of pick, yes or no — and each had its own
//! button component and its own handler, which matched the pending decision
//! and rebuilt the answer the button stood for. A [`DecisionChoice`] carries
//! its answer, built with the modal, and one handler submits it.
//!
//! What stays apart is what isn't one click: the creature-type pair (two
//! picks in a row), the "Always" buttons (they record a standing answer
//! too), the colour keys, and the coin flip and die roll (the answer is
//! rolled when clicked).

use super::*;
use crabomination::decision::LearnChoice;
use crabomination::mana::Color as ManaColor;

/// A button that answers the decision it was built for with `answer`.
#[derive(Component)]
pub struct DecisionChoice {
    /// The decision it answers (`decision_key`): a button still up from the
    /// one before can't answer the next.
    pub key: DecisionKey,
    pub answer: DecisionAnswer,
}

/// How a choice button looks.
#[derive(Clone, Copy)]
pub enum ChoiceLook {
    Primary,
    Info,
    Neutral,
    /// Declining, naming nothing: the quiet way out.
    Quiet,
    /// A colour of its own, and the text colour that reads on it.
    Paint(Color, Color),
}

/// Spawn a button under `parent` that answers `key`'s decision with
/// `answer`. A cost in `label` draws as mana pips.
fn spawn_choice(
    parent: &mut ChildSpawnerCommands,
    ui_fonts: &UiFonts,
    key: &DecisionKey,
    label: &str,
    answer: DecisionAnswer,
    look: ChoiceLook,
) {
    let (bg, text) = match look {
        ChoiceLook::Primary => (theme::BUTTON_PRIMARY_BG, theme::TEXT_PRIMARY),
        ChoiceLook::Info => (theme::BUTTON_INFO_BG, theme::TEXT_PRIMARY),
        ChoiceLook::Neutral => (theme::BUTTON_NEUTRAL_BG, theme::TEXT_PRIMARY),
        ChoiceLook::Quiet => (theme::BUTTON_TERTIARY_BG, theme::TEXT_SECONDARY),
        ChoiceLook::Paint(bg, text) => (bg, text),
    };
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(9.0)),
                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(bg),
            HoverTint::new(bg),
            DecisionChoice { key: key.clone(), answer },
        ))
        .with_children(|b| {
            crate::mana_text::spawn_text(b, ui_fonts, label, 14.0, text).insert(Pickable::IGNORE);
        });
}

/// A column of choices under a title, stretched to the panel's width.
fn column(p: &mut ChildSpawnerCommands, f: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(8.0), align_self: AlignSelf::Stretch, ..default() })
        .with_children(f);
}

/// A row of choices.
fn row(p: &mut ChildSpawnerCommands, f: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() }).with_children(f);
}

fn title(p: &mut ChildSpawnerCommands, ui_fonts: &UiFonts, text: &str) {
    crate::mana_text::spawn_text(p, ui_fonts, text, 18.0, theme::TEXT_PRIMARY);
}

fn note(p: &mut ChildSpawnerCommands, ui_fonts: &UiFonts, text: &str) {
    p.spawn((Text::new(text), ui_fonts.tf(13.0), TextColor(theme::TEXT_SECONDARY)));
}

/// Submit the answer a clicked [`DecisionChoice`] carries, if the decision
/// it was built for is still the one pending.
pub fn handle_decision_choices(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    buttons: Query<(&Interaction, &DecisionChoice), Changed<Interaction>>,
) {
    let Some(cv) = &view.0 else { return };
    let Some(pending) = cv.pending_decision.as_ref().filter(|pd| pd.acting_player == cv.your_seat) else { return };
    let Some(current) = pending.decision.as_ref().and_then(decision_key) else { return };
    let Some(outbox) = outbox else { return };
    let Some((_, choice)) = buttons.iter().find(|(i, c)| **i == Interaction::Pressed && c.key == current) else {
        return;
    };
    outbox.submit(GameAction::SubmitDecision(choice.answer.clone()));
    state.spawned_for = None;
}

/// "Choose a color": one painted button per legal colour (keys W U B R G
/// answer too, `handle_color_keys`).
pub(super) fn spawn_choose_color_modal(commands: &mut Commands, ui_fonts: &UiFonts, key: &DecisionKey, legal: &[ManaColor]) {
    let panel = spawn_modal_panel(commands, 0.0);
    commands.entity(panel).with_children(|p| {
        title(p, ui_fonts, "Choose a color");
        row(p, |r| {
            for c in legal {
                let (bg, label) = match c {
                    ManaColor::White => (Color::srgb(0.92, 0.92, 0.78), "White (W)"),
                    ManaColor::Blue => (Color::srgb(0.30, 0.55, 0.90), "Blue (U)"),
                    ManaColor::Black => (Color::srgb(0.20, 0.20, 0.24), "Black (B)"),
                    ManaColor::Red => (Color::srgb(0.85, 0.28, 0.20), "Red (R)"),
                    ManaColor::Green => (Color::srgb(0.25, 0.60, 0.30), "Green (G)"),
                };
                let text = if *c == ManaColor::White { Color::BLACK } else { Color::WHITE };
                spawn_choice(r, ui_fonts, key, label, DecisionAnswer::Color(*c), ChoiceLook::Paint(bg, text));
            }
        });
    });
}

/// The colour keys: W U B R G answer a pending "choose a color" with that
/// colour, when it's on offer. Not while a text surface owns the keyboard
/// (typing a chat line must not pick a colour).
pub fn handle_color_keys(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    text_input: crate::systems::input_guard::TextInputGuard,
) {
    let Some(cv) = &view.0 else { return };
    let Some(DecisionWire::ChooseColor { legal, .. }) = cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref())
    else {
        return;
    };
    let Some(outbox) = outbox else { return };
    if text_input.typing() {
        return;
    }
    let keys = [
        (KeyCode::KeyW, ManaColor::White),
        (KeyCode::KeyU, ManaColor::Blue),
        (KeyCode::KeyB, ManaColor::Black),
        (KeyCode::KeyR, ManaColor::Red),
        (KeyCode::KeyG, ManaColor::Green),
    ];
    if let Some((_, c)) = keys.into_iter().find(|(k, c)| keyboard.just_pressed(*k) && legal.contains(c)) {
        outbox.submit(GameAction::SubmitDecision(DecisionAnswer::Color(c)));
        state.spawned_for = None;
    }
}

/// A modal trigger's "choose one": a button per mode.
pub(super) fn spawn_choose_trigger_mode_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    key: &DecisionKey,
    heading: &str,
    num_modes: usize,
    mode_texts: &[String],
) {
    let panel = spawn_modal_panel(commands, 380.0);
    commands.entity(panel).with_children(|p| {
        title(p, ui_fonts, &format!("{heading} — choose one"));
        column(p, |c| {
            for idx in 0..num_modes {
                spawn_choice(c, ui_fonts, key, &mode_label(idx, mode_texts), DecisionAnswer::Mode(idx), ChoiceLook::Primary);
            }
        });
    });
}

/// CR 701.38 ballot — a button per printed option, answered by its index.
pub(super) fn spawn_option_ballot_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    key: &DecisionKey,
    heading: &str,
    options: &[String],
) {
    let panel = spawn_modal_panel(commands, 380.0);
    fill_option_ballot(commands, panel, ui_fonts, key, heading, options);
}

/// The ballot's title and one button per option, under `panel` (the layout
/// harness fills a panel of its own: a `DecisionModal` with no decision
/// pending is despawned).
pub(crate) fn fill_option_ballot(
    commands: &mut Commands,
    panel: Entity,
    ui_fonts: &UiFonts,
    key: &DecisionKey,
    heading: &str,
    options: &[String],
) {
    commands.entity(panel).with_children(|p| {
        title(p, ui_fonts, heading);
        column(p, |c| {
            for (idx, label) in options.iter().enumerate() {
                spawn_choice(c, ui_fonts, key, label, DecisionAnswer::Amount(idx as u32), ChoiceLook::Primary);
            }
        });
    });
}

/// CR 704.5j — which of several same-named legends survives the legend
/// rule: a button per duplicate.
pub(super) fn spawn_legend_keep_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    key: &DecisionKey,
    name: &str,
    duplicates: &[(CardId, String)],
) {
    let panel = spawn_modal_panel(commands, 360.0);
    commands.entity(panel).with_children(|p| {
        title(p, ui_fonts, &format!("Legend rule: choose which \"{name}\" to keep"));
        note(p, ui_fonts, "The others will be put into your graveyard.");
        column(p, |c| {
            for (i, (id, label)) in duplicates.iter().enumerate() {
                spawn_choice(c, ui_fonts, key, &format!("#{} — {label}", i + 1), DecisionAnswer::KeptLegend(*id), ChoiceLook::Neutral);
            }
        });
    });
}

/// CR 201.3 — name a card: up to eight suggestions, or nothing.
pub(super) fn spawn_name_card_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    key: &DecisionKey,
    source_name: &str,
    suggestions: &[String],
    restriction: Option<&str>,
) {
    let panel = spawn_modal_panel(commands, 280.0);
    commands.entity(panel).with_children(|p| {
        // CR 201.4a — say which namespace the choice is restricted to.
        let kind = restriction.map(|r| format!("{r} ")).unwrap_or_default();
        title(p, ui_fonts, &format!("{source_name} — choose a {kind}card name"));
        column(p, |c| {
            for name in suggestions.iter().take(8) {
                spawn_choice(c, ui_fonts, key, name, DecisionAnswer::NamedCard(name.clone()), ChoiceLook::Info);
            }
            spawn_choice(c, ui_fonts, key, "Name nothing", DecisionAnswer::NamedCard(String::new()), ChoiceLook::Quiet);
        });
    });
}

/// Learn — reveal a Lesson into hand, discard a card to draw, or neither.
pub(super) fn spawn_learn_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    key: &DecisionKey,
    lessons: &[(CardId, String)],
    hand: &[(CardId, String)],
) {
    let panel = spawn_modal_panel(commands, 280.0);
    let learn = |choice| DecisionAnswer::Learn(choice);
    commands.entity(panel).with_children(|p| {
        title(p, ui_fonts, "Learn — reveal a Lesson, or discard a card to draw");
        note(p, ui_fonts, "Reveal a Lesson into your hand:");
        column(p, |c| {
            let look = ChoiceLook::Paint(Color::srgb(0.25, 0.45, 0.30), Color::WHITE);
            for (id, name) in lessons {
                spawn_choice(c, ui_fonts, key, name, learn(LearnChoice::FetchLesson(*id)), look);
            }
        });
        if !hand.is_empty() {
            note(p, ui_fonts, "Or discard a card to draw:");
            column(p, |c| {
                let look = ChoiceLook::Paint(Color::srgb(0.40, 0.30, 0.20), Color::WHITE);
                for (id, name) in hand {
                    let answer = learn(LearnChoice::Rummage { discard: *id });
                    spawn_choice(c, ui_fonts, key, &format!("Discard {name}"), answer, look);
                }
            });
        }
        column(p, |c| spawn_choice(c, ui_fonts, key, "Decline", learn(LearnChoice::Decline), ChoiceLook::Quiet));
    });
}

/// A yes/no ask (`OptionalTrigger`, and CR 903.9b's commander redirect),
/// with — where `show_always` — the standing-answer row under it.
pub(super) fn spawn_optional_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    key: &DecisionKey,
    description: &str,
    show_always: bool,
) {
    let panel = spawn_modal_panel(commands, 0.0);
    commands.entity(panel).insert(Node {
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(20.0)),
        row_gap: Val::Px(16.0),
        align_items: AlignItems::Center,
        max_width: Val::Percent(70.0),
        border_radius: BorderRadius::all(theme::RADIUS_PANEL),
        ..default()
    });
    commands.entity(panel).with_children(|p| {
        // "Pay {3} to keep this trigger?" — its cost as mana pips.
        crate::mana_text::spawn_text(p, ui_fonts, description, 16.0, theme::TEXT_PRIMARY);
        row(p, |r| {
            spawn_choice(r, ui_fonts, key, "Yes", DecisionAnswer::Bool(true), ChoiceLook::Primary);
            spawn_choice(r, ui_fonts, key, "No", DecisionAnswer::Bool(false), ChoiceLook::Info);
        });
        // Standing-answer row: smaller, tertiary styling, so the one-shot
        // Yes/No stays the visually primary choice.
        if show_always {
            row(p, |r| {
                for (answer, label) in [(true, "Always Yes"), (false, "Always No")] {
                    r.spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                            border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                            ..default()
                        },
                        BackgroundColor(theme::BUTTON_TERTIARY_BG),
                        HoverTint::new(theme::BUTTON_TERTIARY_BG),
                        OptionalAlwaysButton(answer),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new(label), ui_fonts.tf(13.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE));
                    });
                }
            });
            p.spawn((
                Text::new("Always = auto-answer this prompt for the rest of the game"),
                ui_fonts.tf(11.0),
                TextColor(theme::TEXT_MUTED),
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination::net::{ClientMsg, PendingDecisionView};

    fn app_with(decision: DecisionWire) -> (App, std::sync::mpsc::Receiver<ClientMsg>) {
        let mut g = crabomination::game::two_player_game();
        g.priority.player_with_priority = 0;
        let mut cv = crabomination::server::view::project(&g, 0);
        cv.pending_decision = Some(PendingDecisionView { acting_player: 0, decision: Some(decision), cancellable: false });
        let (tx, rx) = std::sync::mpsc::channel();
        let mut app = App::new();
        app.insert_resource(CurrentView(Some(cv)))
            .insert_resource(DecisionUiState::default())
            .insert_resource(NetOutbox::new(tx))
            .add_systems(Update, handle_decision_choices);
        (app, rx)
    }

    /// A clicked choice submits the answer it carries — and only for the
    /// decision it was built for: a button left over from the one before
    /// answers nothing.
    #[test]
    fn a_choice_answers_its_own_decision_only() {
        let decision = DecisionWire::ChooseColor { source: CardId(7), legal: vec![ManaColor::Red, ManaColor::Green] };
        let key = decision_key(&decision).unwrap();
        let (mut app, rx) = app_with(decision);
        let stale = DecisionKey::ChooseColor(CardId(8));
        app.world_mut().spawn((Interaction::Pressed, DecisionChoice { key: stale, answer: DecisionAnswer::Color(ManaColor::Blue) }));
        app.update();
        assert!(rx.try_recv().is_err(), "a stale button answers nothing");

        app.world_mut().spawn((Interaction::Pressed, DecisionChoice { key, answer: DecisionAnswer::Color(ManaColor::Green) }));
        app.update();
        match rx.try_recv() {
            Ok(ClientMsg::SubmitAction(GameAction::SubmitDecision(DecisionAnswer::Color(c)))) => assert_eq!(c, ManaColor::Green),
            other => panic!("expected the green answer, got {other:?}"),
        }
    }
}
