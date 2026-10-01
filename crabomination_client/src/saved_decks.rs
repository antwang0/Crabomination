//! Your decks: every decklist that imports is kept, and the menu lists them.
//!
//! A list read "From File" or "From Clipboard" was gone the moment the match
//! started; playing it again meant finding the file or copying it again.
//! Each one that imports is now saved as a plain text file under the config
//! directory (`<config>/crabomination/decks/<name>.txt`), named for its
//! commanders, the name an Arena export carries, or the file it came from.
//! "Your Decks" in the menu lists them — a click plays one in the selected
//! format — and the Commander deck picker deals a saved Commander list to
//! any seat, bots included. A list dropped into the folder by hand shows up
//! the same way. The same cards imported twice are saved once.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use crabomination::cube::CardFactory;

use crate::deck_import::{ImportReport, import_deck};
use crate::menu::{AppState, MenuFields, MenuRoot, MenuStatus, NetMode, PendingNetMode};
use crate::theme::{
    self, BUTTON_DANGER_BG, BUTTON_INFO_BG, BUTTON_NEUTRAL_BG, FIELD_BG, HoverTint, RADIUS_BUTTON, RADIUS_PANEL, UiFonts,
};

/// One saved decklist.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SavedDeck {
    /// The file's stem, which is how the menu names it.
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) text: String,
}

impl SavedDeck {
    /// The row's detail: "Atraxa, Praetors' Voice · 100 cards", "60 cards",
    /// "60 cards · 2 not in the catalog" — the commanders only when the
    /// deck isn't already named for them.
    pub(crate) fn summary(&self) -> String {
        let parsed = crabomination::decklist::parse_decklist(&self.text);
        let cards = parsed.main.len() + parsed.commanders.len();
        let mut parts = Vec::new();
        let commanders = parsed.commanders.iter().map(|f| f().name).collect::<Vec<_>>().join(" + ");
        if !commanders.is_empty() && commanders != self.name {
            parts.push(commanders);
        }
        parts.push(format!("{cards} cards"));
        if !parsed.unknown.is_empty() {
            parts.push(format!("{} not in the catalog", parsed.unknown.len()));
        }
        parts.join(" · ")
    }

    /// The deck as a Commander pod seat — `(commanders, main)` — when it is
    /// a legal Commander list.
    pub(crate) fn commander_seat(&self) -> Option<(Vec<CardFactory>, Vec<CardFactory>)> {
        let parsed = crabomination::decklist::parse_decklist(&self.text);
        if !parsed.unknown.is_empty() {
            return None;
        }
        parsed.commander_list().ok().map(|l| (l.commanders, l.main))
    }
}

/// Where the decks live: `<config>/crabomination/decks`.
#[cfg(not(target_arch = "wasm32"))]
fn decks_dir() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("crabomination").join("decks"))
}

/// The browser build has no deck folder (and no decklist import).
#[cfg(target_arch = "wasm32")]
fn decks_dir() -> Option<PathBuf> {
    None
}

/// Every saved deck, by name in any case.
pub(crate) fn load_all() -> Vec<SavedDeck> {
    decks_dir().map(|d| load_in(&d)).unwrap_or_default()
}

/// The saved deck called `name`.
pub(crate) fn find(name: &str) -> Option<SavedDeck> {
    load_all().into_iter().find(|d| d.name == name)
}

/// Keep `text`, an imported list, unless the same cards are already saved.
/// `file` is the path it was read from, when it came from one. Returns the
/// name it is saved under.
pub(crate) fn save(text: &str, file: Option<&Path>) -> std::io::Result<String> {
    let Some(dir) = decks_dir() else {
        return Err(std::io::Error::other("no config directory"));
    };
    save_in(&dir, text, file)
}

/// Delete a saved deck's file.
pub(crate) fn delete(deck: &SavedDeck) -> std::io::Result<()> {
    std::fs::remove_file(&deck.path)
}

fn load_in(dir: &Path) -> Vec<SavedDeck> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut decks: Vec<SavedDeck> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "txt"))
        .filter_map(|path| {
            let name = path.file_stem()?.to_str()?.to_string();
            let text = std::fs::read_to_string(&path).ok()?;
            Some(SavedDeck { name, path, text })
        })
        .collect();
    decks.sort_by_key(|d| d.name.to_lowercase());
    decks
}

fn save_in(dir: &Path, text: &str, file: Option<&Path>) -> std::io::Result<String> {
    let key = card_key(text);
    let existing = load_in(dir);
    if let Some(same) = existing.iter().find(|d| card_key(&d.text) == key) {
        return Ok(same.name.clone());
    }
    let base = file_safe(&suggested_name(text, file));
    let name = std::iter::once(base.clone())
        .chain((2..).map(|n| format!("{base} ({n})")))
        .find(|n| !existing.iter().any(|d| d.name.eq_ignore_ascii_case(n)))
        .expect("an unused name");
    std::fs::create_dir_all(dir)?;
    // Written whole and then renamed, so a crash mid-write can't leave a
    // half list for the menu to offer.
    let path = dir.join(format!("{name}.txt"));
    let part = dir.join(format!("{name}.txt.part"));
    std::fs::write(&part, text)?;
    std::fs::rename(&part, &path)?;
    Ok(name)
}

/// The cards a list names, commanders marked, in a fixed order — two lists
/// with the same key are the same deck however they were written.
fn card_key(text: &str) -> Vec<(bool, &'static str)> {
    let parsed = crabomination::decklist::parse_decklist(text);
    let mut key: Vec<(bool, &'static str)> = parsed
        .commanders
        .iter()
        .map(|f| (true, f().name))
        .chain(parsed.main.iter().chain(&parsed.sideboard).map(|f| (false, f().name)))
        .collect();
    key.sort_unstable();
    key
}

/// What to call a list: the name an Arena export's `About` header gives it,
/// else its commanders, else the file it came from (unless that is the
/// menu's default `deck.txt`), else its most-played spell.
fn suggested_name(text: &str, file: Option<&Path>) -> String {
    let mut lines = text.lines().map(str::trim);
    if lines.by_ref().any(|l| l.eq_ignore_ascii_case("about"))
        && let Some(name) = lines.take_while(|l| !l.is_empty()).find_map(|l| l.strip_prefix("Name "))
        && !name.trim().is_empty()
    {
        return name.trim().to_string();
    }
    let parsed = crabomination::decklist::parse_decklist(text);
    let commanders = match parsed.commander_list() {
        Ok(list) => list.commanders,
        Err(_) => parsed.commanders.clone(),
    };
    if !commanders.is_empty() {
        return commanders.iter().map(|f| f().name).collect::<Vec<_>>().join(" + ");
    }
    if let Some(stem) = file.and_then(|f| f.file_stem()).and_then(|s| s.to_str())
        && !["deck", "decklist", "list"].contains(&stem.to_lowercase().as_str())
    {
        return stem.to_string();
    }
    let mut counts: Vec<(&'static str, usize)> = Vec::new();
    for def in parsed.main.iter().map(|f| f()).filter(|d| !d.is_land()) {
        match counts.iter_mut().find(|(n, _)| *n == def.name) {
            Some((_, c)) => *c += 1,
            None => counts.push((def.name, 1)),
        }
    }
    // The first listed of the most-played, so a list names the same way
    // every time.
    let top = counts.iter().rev().max_by_key(|(_, c)| *c).map(|(n, _)| *n);
    match top {
        Some(card) => format!("{card} deck"),
        None => "Imported deck".to_string(),
    }
}

/// `name` as a file name every desktop accepts: no path separators or
/// reserved characters, no leading dot, not empty, not too long.
fn file_safe(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_control() || "/\\:*?\"<>|".contains(c) { '_' } else { c })
        .collect();
    let cleaned = cleaned.trim().trim_start_matches('.').trim();
    let cleaned: String = cleaned.chars().take(80).collect();
    if cleaned.is_empty() { "Imported deck".to_string() } else { cleaned }
}

// ── The "Your Decks" overlay ────────────────────────────────────────────────

/// "Your Decks" in the menu's decklist section.
#[derive(Component)]
pub(crate) struct SavedDecksButton;

/// The overlay's state: open or not, the decks it lists, and the deck whose
/// Delete was pressed once (a second press deletes it).
#[derive(Resource, Default)]
pub(crate) struct SavedDecksPanel {
    open: bool,
    decks: Vec<SavedDeck>,
    confirm_delete: Option<usize>,
}

#[derive(Component)]
pub(crate) struct SavedDecksRoot;

#[derive(Component)]
pub(crate) struct PlayDeckRow(usize);

#[derive(Component)]
pub(crate) struct DeleteDeckButton(usize);

#[derive(Component)]
pub(crate) struct CloseDecksButton;

impl SavedDecksPanel {
    /// Open over the decks on disk now.
    pub(crate) fn open(&mut self) {
        *self = SavedDecksPanel { open: true, decks: load_all(), confirm_delete: None };
    }
}

/// "Your Decks" opens the overlay.
pub(crate) fn open_saved_decks(
    q: Query<&Interaction, (Changed<Interaction>, With<SavedDecksButton>)>,
    mut panel: ResMut<SavedDecksPanel>,
    mut fields: ResMut<MenuFields>,
) {
    if q.iter().any(|i| *i == Interaction::Pressed) {
        panel.open();
        fields.blur();
    }
}

/// Leaving the menu closes the overlay (it goes with the menu).
pub(crate) fn close_saved_decks(mut panel: ResMut<SavedDecksPanel>) {
    panel.open = false;
}

/// Rebuild the overlay whenever what it shows changes.
pub(crate) fn sync_saved_decks(
    mut commands: Commands,
    panel: Res<SavedDecksPanel>,
    fields: Res<MenuFields>,
    ui_fonts: Res<UiFonts>,
    roots: Query<Entity, With<SavedDecksRoot>>,
) {
    if !panel.is_changed() && !fields.is_changed() {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    if panel.open {
        let tf = |size: f32| ui_fonts.tf(size);
        spawn_panel(&mut commands, &tf, &panel, fields.format.label());
    }
}

fn spawn_panel(commands: &mut Commands, tf: &impl Fn(f32) -> TextFont, panel: &SavedDecksPanel, format: &str) {
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
            SavedDecksRoot,
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
                BorderColor::all(theme::ACCENT_GOLD),
            ))
            .with_children(|p| {
                p.spawn((Text::new("Your decks"), tf(22.0), TextColor(theme::ACCENT_GOLD)));
                let note = if panel.decks.is_empty() {
                    "None yet — a decklist you import From File or From Clipboard is kept here.".to_string()
                } else {
                    format!("Click a deck to play it as {format}.")
                };
                p.spawn((Text::new(note), tf(13.0), TextColor(theme::TEXT_BODY)));
                p.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        max_height: Val::Px(420.0),
                        min_height: Val::Px(0.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    crate::systems::scroll::Scrollable::with_line_px(28.0),
                ))
                .with_children(|list| {
                    for (i, deck) in panel.decks.iter().enumerate() {
                        deck_row(list, tf, i, deck, panel.confirm_delete == Some(i));
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
                    let folder = decks_dir().map(|d| d.display().to_string()).unwrap_or_default();
                    row.spawn((Text::new(format!("{folder} · Esc closes")), tf(11.0), TextColor(theme::TEXT_MUTED)));
                    row.spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)),
                            border_radius: BorderRadius::all(RADIUS_BUTTON),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        BackgroundColor(BUTTON_INFO_BG),
                        HoverTint::new(BUTTON_INFO_BG),
                        CloseDecksButton,
                        children![(Text::new("Close"), tf(15.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE)],
                    ));
                });
            });
        });
}

/// A deck's row: its name and summary to click and play, and a Delete that
/// asks once.
fn deck_row(l: &mut ChildSpawnerCommands, tf: &impl Fn(f32) -> TextFont, i: usize, deck: &SavedDeck, confirming: bool) {
    l.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(6.0), flex_shrink: 0.0, ..default() })
        .with_children(|row| {
            row.spawn((
                Button,
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    border_radius: BorderRadius::all(RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(FIELD_BG),
                HoverTint::new(FIELD_BG),
                PlayDeckRow(i),
            ))
            .with_children(|b| {
                b.spawn((Text::new(deck.name.clone()), tf(13.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE));
                b.spawn((Text::new(deck.summary()), tf(11.0), TextColor(theme::TEXT_MUTED), Pickable::IGNORE));
            });
            let (label, bg) = if confirming { ("Delete?", BUTTON_DANGER_BG) } else { ("Delete", BUTTON_NEUTRAL_BG) };
            row.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    border_radius: BorderRadius::all(RADIUS_BUTTON),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    width: Val::Px(76.0),
                    ..default()
                },
                BackgroundColor(bg),
                HoverTint::new(bg),
                DeleteDeckButton(i),
                children![(Text::new(label), tf(12.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE)],
            ));
        });
}

/// A row plays its deck in the menu's format — or, when the deck can't play
/// there, closes the overlay on the import report saying why. Delete asks
/// once and then removes the file; Close or Esc closes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_saved_decks(
    mut commands: Commands,
    mut panel: ResMut<SavedDecksPanel>,
    mut next_state: ResMut<NextState<AppState>>,
    mut pending: ResMut<PendingNetMode>,
    mut status: ResMut<MenuStatus>,
    mut report: ResMut<ImportReport>,
    fields: Res<MenuFields>,
    mut keys: MessageReader<KeyboardInput>,
    play_q: Query<(&Interaction, &PlayDeckRow), Changed<Interaction>>,
    delete_q: Query<(&Interaction, &DeleteDeckButton), Changed<Interaction>>,
    close_q: Query<&Interaction, (Changed<Interaction>, With<CloseDecksButton>)>,
) {
    let esc = keys.read().any(|k| k.state.is_pressed() && k.logical_key == Key::Escape);
    if !panel.open {
        return;
    }
    if esc || close_q.iter().any(|i| *i == Interaction::Pressed) {
        panel.open = false;
        return;
    }
    for (interaction, del) in &delete_q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if panel.confirm_delete != Some(del.0) {
            panel.confirm_delete = Some(del.0);
            continue;
        }
        if let Some(deck) = panel.decks.get(del.0) {
            match delete(deck) {
                Ok(()) => status.0 = format!("Deleted \"{}\".", deck.name),
                Err(e) => status.0 = format!("Can't delete \"{}\": {e}", deck.name),
            }
        }
        panel.decks = load_all();
        panel.confirm_delete = None;
    }
    let Some(deck) = play_q
        .iter()
        .find(|(i, _)| **i == Interaction::Pressed)
        .and_then(|(_, row)| panel.decks.get(row.0))
        .cloned()
    else {
        return;
    };
    match import_deck(&deck.text, fields.format) {
        Ok(imported) => {
            status.0.clear();
            commands.insert_resource(imported);
            pending.0 = Some((NetMode::LocalBot, fields.format));
            next_state.set(AppState::InGame);
        }
        Err(problems) => {
            status.0 = format!("\"{}\": {}", deck.name, problems.summary);
            report.0 = (!problems.lines.is_empty()).then_some(problems);
            panel.open = false;
        }
    }
}

/// A shared saved-deck name, as a Commander seat's choice holds it.
pub(crate) type DeckName = Arc<str>;

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("crab-saved-decks-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    const BURN: &str = "20 Mountain\n4 Lightning Bolt\n4 Lava Spike\n4 Rift Bolt\n4 Chain Lightning\n";

    /// An import is saved once however often it comes back, named for what
    /// it is, and a different list under a taken name gets a numbered one.
    #[test]
    fn an_imported_list_is_saved_once_under_a_name_that_fits() {
        let dir = scratch_dir("save");
        assert_eq!(save_in(&dir, BURN, None).unwrap(), "Lightning Bolt deck");
        // Same cards, written another way: no second copy.
        let reordered = "4 Chain Lightning\n4x Rift Bolt\n4 Lava Spike\n4 Lightning Bolt\n20 Mountain\n";
        assert_eq!(save_in(&dir, reordered, None).unwrap(), "Lightning Bolt deck");
        assert_eq!(load_in(&dir).len(), 1);
        // A different list whose best name is taken.
        let more = format!("{BURN}4 Lightning Bolt\n");
        assert_eq!(save_in(&dir, &more, None).unwrap(), "Lightning Bolt deck (2)");
        // A file's own name; the menu's default deck.txt says nothing.
        let mono = "40 Forest\n";
        assert_eq!(save_in(&dir, mono, Some(Path::new("/tmp/Green Stompy.txt"))).unwrap(), "Green Stompy");
        let names: Vec<String> = load_in(&dir).into_iter().map(|d| d.name).collect();
        assert_eq!(names, ["Green Stompy", "Lightning Bolt deck", "Lightning Bolt deck (2)"]);
        assert_eq!(load_in(&dir)[1].text, BURN, "saved as it was imported");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_list_is_named_by_its_header_or_its_commanders() {
        let arena = "About\nName Izzet: Burn/Tempo?\n\nDeck\n4 Lightning Bolt\n";
        assert_eq!(suggested_name(arena, None), "Izzet: Burn/Tempo?");
        assert_eq!(file_safe(&suggested_name(arena, None)), "Izzet_ Burn_Tempo_");
        let cmdr = "Commander\n1 Atraxa, Praetors' Voice\n\nDeck\n1 Sol Ring\n";
        assert_eq!(suggested_name(cmdr, Some(Path::new("deck.txt"))), "Atraxa, Praetors' Voice");
        assert_eq!(suggested_name("40 Forest\n", Some(Path::new("deck.txt"))), "Imported deck");
        assert_eq!(file_safe(" ..hidden "), "hidden");
    }

    /// A stock Commander list written out and read back is a pod seat; a
    /// 60-card list isn't.
    #[test]
    fn a_saved_commander_list_is_a_pod_seat() {
        let stock = crabomination::pod::target_decks()[4];
        let mut text = String::from("Commander\n");
        for f in stock.commanders {
            text.push_str(&format!("1 {}\n", f().name));
        }
        text.push_str("\nDeck\n");
        for f in stock.main {
            text.push_str(&format!("1 {}\n", f().name));
        }
        let mut deck = SavedDeck { name: "Krark".into(), path: PathBuf::new(), text };
        let (commanders, main) = deck.commander_seat().expect("a legal list");
        assert_eq!(commanders.len(), 2);
        assert_eq!(commanders.len() + main.len(), 100);
        assert_eq!(deck.summary(), "Krark, the Thumbless + Rograkh, Son of Rohgahh · 100 cards");
        deck.name = "Krark, the Thumbless + Rograkh, Son of Rohgahh".into();
        assert_eq!(deck.summary(), "100 cards", "not the name twice");
        let burn = SavedDeck { name: "Burn".into(), path: PathBuf::new(), text: BURN.into() };
        assert!(burn.commander_seat().is_none());
        assert_eq!(burn.summary(), "36 cards");
    }
}
