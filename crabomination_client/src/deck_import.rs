//! Bring your own deck to a local match, and say in full why one can't play.
//!
//! The menu's decklist buttons: "From File" reads the file named in its Deck
//! file field; "From Clipboard" reads the clipboard, so a list copied off Moxfield,
//! Archidekt or an Arena export needs no file. Either way the deck has to be
//! all catalog cards and legal for the menu's format ([`import_deck`]). When
//! it isn't, the status line says so in brief and the import report lists
//! every problem — each unknown card with the names it probably meant — with
//! a button that copies the list. The status line used to name four unknown
//! cards and stop, so a list with forty had to be fixed four at a time.
//! A list that imports is kept for next time ([`crate::saved_decks`]).

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crate::menu::{
    AppState, ImportedDeck, MatchFormat, MenuFields, MenuRoot, MenuStatus, NetMode, PendingNetMode,
};
use crate::theme::{self, HoverTint, UiFonts, BUTTON_INFO_BG, BUTTON_NEUTRAL_BG, RADIUS_BUTTON, RADIUS_PANEL};

/// Why a decklist can't be played: the menu's status line, and every
/// problem one per line for the import report (empty when the one line
/// says it all, like an unreadable file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportProblems {
    pub(crate) summary: String,
    /// The report's heading over `lines`.
    pub(crate) heading: String,
    pub(crate) lines: Vec<String>,
}

impl ImportProblems {
    fn one_line(summary: impl Into<String>) -> Self {
        Self { summary: summary.into(), heading: String::new(), lines: Vec::new() }
    }

    fn listed<E: std::fmt::Display>(heading: String, errs: &[E]) -> Self {
        Self {
            summary: crabomination::format::error_summary(errs, 2),
            heading,
            lines: errs.iter().map(ToString::to_string).collect(),
        }
    }
}

/// Read `text` as the deck the player brings to a `format` match against
/// the bot: every card in the catalog, and legal for the format — a
/// Commander list by CR 903's rules, the rest by their constructed or
/// limited deck rules.
pub(crate) fn import_deck(text: &str, format: MatchFormat) -> Result<ImportedDeck, ImportProblems> {
    let parsed = crabomination::decklist::parse_decklist(text);
    if !parsed.unknown.is_empty() {
        let n = parsed.unknown.len();
        let cards = if n == 1 { "card isn't" } else { "cards aren't" };
        let mut problems = ImportProblems::listed(format!("{n} {cards} in the catalog yet:"), &parsed.unknown);
        problems.summary = format!("{n} {cards} in the catalog: {}", problems.summary);
        return Err(problems);
    }
    if parsed.main.is_empty() && parsed.commanders.is_empty() {
        return Err(ImportProblems::one_line("No cards found — is that a decklist?"));
    }
    if format == MatchFormat::Commander {
        // CR 903 — the commander section, pair, identity and 100-card
        // singleton rules, all checked by the engine.
        return match parsed.commander_list() {
            Ok(list) => Ok(ImportedDeck { main: list.main, commanders: list.commanders }),
            Err(errs) => Err(ImportProblems::listed(illegal(format), &errs)),
        };
    }
    let rules = match format {
        MatchFormat::Modern => crabomination::format::Format::Modern,
        MatchFormat::Commander => crabomination::format::Format::Commander,
        // Cube / SoS pools and sealed lists play limited-style 40-card rules.
        MatchFormat::Cube | MatchFormat::Sos | MatchFormat::Sealed => crabomination::format::Format::Draft,
    };
    let main: Vec<_> = parsed.main.iter().map(|f| f()).collect();
    match crabomination::format::validate_deck(&main, rules) {
        Ok(()) => Ok(ImportedDeck { main: parsed.main, commanders: Vec::new() }),
        Err(errs) => Err(ImportProblems::listed(illegal(format), &errs)),
    }
}

fn illegal(format: MatchFormat) -> String {
    format!("It isn't a legal {} deck:", format.label())
}

/// Decklist "From File" — reads the decklist file named in the Deck file
/// field and starts a local-bot match with it.
#[derive(Component)]
pub(crate) struct ImportDeckButton;

/// Decklist "From Clipboard" — the same, reading the clipboard.
#[derive(Component)]
pub(crate) struct PasteDeckButton;

/// The import report on screen, if any.
#[derive(Resource, Default)]
pub(crate) struct ImportReport(pub(crate) Option<ImportProblems>);

#[derive(Component)]
pub(crate) struct ImportReportRoot;

#[derive(Component)]
pub(crate) struct CopyReportButton;

#[derive(Component)]
pub(crate) struct CopyReportText;

#[derive(Component)]
pub(crate) struct CloseReportButton;

/// Read the deck a button asks for and start the match, or report why not.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_import_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut pending: ResMut<PendingNetMode>,
    mut status: ResMut<MenuStatus>,
    mut report: ResMut<ImportReport>,
    mut clipboard: ResMut<bevy::clipboard::Clipboard>,
    fields: Res<MenuFields>,
    import_q: Query<&Interaction, (Changed<Interaction>, With<ImportDeckButton>)>,
    paste_q: Query<&Interaction, (Changed<Interaction>, With<PasteDeckButton>)>,
) {
    let mut file = None;
    let text = if import_q.iter().any(|i| *i == Interaction::Pressed) {
        let path = fields.deck_path.trim();
        file = Some(std::path::PathBuf::from(path));
        match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) => {
                status.0 = format!("Can't read {path}: {e}");
                return;
            }
        }
    } else if paste_q.iter().any(|i| *i == Interaction::Pressed) {
        // Desktop reads resolve at once; this group is native-only.
        match clipboard.fetch_text().poll_result() {
            Some(Ok(text)) if !text.trim().is_empty() => text,
            _ => {
                status.0 = "The clipboard has no text — copy a decklist first.".into();
                return;
            }
        }
    } else {
        return;
    };
    match import_deck(&text, fields.format) {
        Ok(deck) => {
            // Kept for "Your Decks" (`saved_decks`); a failed save costs
            // only the copy.
            if let Err(e) = crate::saved_decks::save(&text, file.as_deref()) {
                eprintln!("saved decks: can't keep the list ({e})");
            }
            status.0.clear();
            commands.insert_resource(deck);
            pending.0 = Some((NetMode::LocalBot, fields.format));
            next_state.set(AppState::InGame);
        }
        Err(problems) => {
            status.0 = problems.summary.clone();
            report.0 = (!problems.lines.is_empty()).then_some(problems);
        }
    }
}

/// Spawn the report when there is one and drop it when it closes.
pub(crate) fn sync_import_report(
    mut commands: Commands,
    report: Res<ImportReport>,
    ui_fonts: Res<UiFonts>,
    roots: Query<Entity, With<ImportReportRoot>>,
) {
    if !report.is_changed() {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    if let Some(problems) = report.0.as_ref() {
        spawn_report(&mut commands, &|size| ui_fonts.tf(size), problems);
    }
}

/// Leaving the menu closes the report (its overlay goes with the menu).
pub(crate) fn close_import_report(mut report: ResMut<ImportReport>) {
    report.0 = None;
}

fn spawn_report(commands: &mut Commands, tf: &impl Fn(f32) -> TextFont, problems: &ImportProblems) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG_HEAVY),
            GlobalZIndex(theme::layer::MODAL),
            ImportReportRoot,
            MenuRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    width: Val::Px(620.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(RADIUS_PANEL),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG_RAISED),
                BorderColor::all(theme::ACCENT_ORANGE),
            ))
            .with_children(|p| {
                p.spawn((Text::new("This deck can't be played"), tf(22.0), TextColor(theme::ACCENT_ORANGE)));
                p.spawn((Text::new(problems.heading.clone()), tf(14.0), TextColor(theme::TEXT_BODY)));
                p.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        max_height: Val::Px(420.0),
                        min_height: Val::Px(0.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    crate::systems::scroll::Scrollable::with_line_px(22.0),
                ))
                .with_children(|list| {
                    for line in &problems.lines {
                        list.spawn((
                            Text::new(line.clone()),
                            tf(13.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Node { flex_shrink: 0.0, ..default() },
                        ));
                    }
                });
                p.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Fix the list and import it again · Esc closes"),
                        tf(11.0),
                        TextColor(theme::TEXT_MUTED),
                    ));
                    row.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(8.0), ..default() })
                        .with_children(|buttons| {
                            report_button(buttons, tf, "Copy list", BUTTON_NEUTRAL_BG, CopyReportButton, Some(CopyReportText));
                            report_button(buttons, tf, "Close", BUTTON_INFO_BG, CloseReportButton, None::<CopyReportText>);
                        });
                });
            });
        });
}

fn report_button<M: Component, T: Component>(
    parent: &mut ChildSpawnerCommands,
    tf: &impl Fn(f32) -> TextFont,
    label: &str,
    bg: Color,
    marker: M,
    text_marker: Option<T>,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(bg),
            HoverTint::new(bg),
            marker,
        ))
        .with_children(|b| {
            let mut text = b.spawn((Text::new(label), tf(15.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE));
            if let Some(m) = text_marker {
                text.insert(m);
            }
        });
}

/// Copy puts the list on the clipboard (heading first, one problem a line);
/// Close or Esc dismisses the report.
pub(crate) fn handle_import_report(
    mut report: ResMut<ImportReport>,
    mut clipboard: ResMut<bevy::clipboard::Clipboard>,
    mut keys: MessageReader<KeyboardInput>,
    copy_q: Query<&Interaction, (Changed<Interaction>, With<CopyReportButton>)>,
    close_q: Query<&Interaction, (Changed<Interaction>, With<CloseReportButton>)>,
    mut copy_text: Query<&mut Text, With<CopyReportText>>,
) {
    let esc = keys.read().any(|k| k.state.is_pressed() && k.logical_key == Key::Escape);
    let Some(problems) = report.0.as_ref() else { return };
    if copy_q.iter().any(|i| *i == Interaction::Pressed) {
        let body = std::iter::once(problems.heading.as_str())
            .chain(problems.lines.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join("\n");
        let label = if clipboard.set_text(body).is_ok() { "Copied" } else { "Can't copy" };
        for mut t in &mut copy_text {
            t.0 = label.into();
        }
    }
    if esc || close_q.iter().any(|i| *i == Interaction::Pressed) {
        report.0 = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every unknown card is listed, each with what it probably meant — not
    /// the first four and "+N more".
    #[test]
    fn an_import_lists_every_unknown_card_with_its_suggestion() {
        let mut text = String::from("4 Lightnig Bolt\n");
        for i in 0..9 {
            text.push_str(&format!("1 Madeup Card Number {i}\n"));
        }
        let Err(problems) = import_deck(&text, MatchFormat::Modern) else { panic!("imported") };
        assert_eq!(problems.heading, "10 cards aren't in the catalog yet:");
        assert_eq!(problems.lines.len(), 10);
        assert_eq!(problems.lines[0], "4x Lightnig Bolt (did you mean Lightning Bolt?)");
        assert!(
            problems.summary.starts_with("10 cards aren't in the catalog: 4x Lightnig Bolt (did you mean"),
            "{}",
            problems.summary,
        );
        assert!(problems.summary.ends_with("(+8 more)"), "{}", problems.summary);
    }

    /// A list of known cards that breaks the format's rules lists each rule
    /// it breaks; one with no cards at all says so on the status line alone.
    #[test]
    fn an_illegal_or_empty_list_says_why() {
        let Err(problems) = import_deck("4 Lightning Bolt\n", MatchFormat::Modern) else { panic!("imported") };
        assert_eq!(problems.heading, "It isn't a legal Modern deck:");
        assert!(!problems.lines.is_empty());

        let Err(problems) = import_deck("Commander\n1 Lightning Bolt\n", MatchFormat::Commander) else {
            panic!("imported")
        };
        assert_eq!(problems.heading, "It isn't a legal Commander deck:");

        let Err(problems) = import_deck("# just a comment\n\n", MatchFormat::Modern) else { panic!("imported") };
        assert!(problems.lines.is_empty());
        assert_eq!(problems.summary, "No cards found — is that a decklist?");
    }

    #[test]
    fn a_legal_list_imports() {
        let text = "20 Mountain\n4 Lightning Bolt\n4 Lava Spike\n4 Rift Bolt\n4 Chain Lightning\n\
                    4 Skewer the Critics\n4 Monastery Swiftspear\n4 Goblin Guide\n4 Eidolon of the Great Revel\n\
                    4 Searing Blaze\n4 Lightning Helix\n";
        let parsed = crabomination::decklist::parse_decklist(text);
        if !parsed.unknown.is_empty() {
            // The fixture leans on a handful of staples; name any the catalog lacks.
            panic!("fixture cards missing: {:?}", parsed.unknown);
        }
        let deck = import_deck(text, MatchFormat::Modern).expect("a legal 60");
        assert_eq!(deck.main.len(), 60);
        assert!(deck.commanders.is_empty());
    }
}
