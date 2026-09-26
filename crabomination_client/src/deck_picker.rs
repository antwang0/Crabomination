//! The Commander deck picker: each seat's stock deck for a local pod.
//!
//! "Decks: …" in the menu's Commander row opens it over the menu: a tab per
//! seat (you, then the pod's bots), a search box, and a scrolling list of
//! every stock deck ([`crabomination::pod::target_decks`]) under a "Random"
//! row. Clicking a row gives the selected seat that deck. The choices live in
//! `MenuFields::pod_decks` and are dealt by `menu::build_local_match_state`;
//! before this, the human always played the first stock deck and the bots'
//! decks cycled one click per deck through the whole field.

use std::sync::OnceLock;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crate::menu::{DeckChoice, MenuFields, MenuRoot, PodDecksButton};
use crate::theme::{
    self, HoverTint, UiFonts, BUTTON_INFO_BG, BUTTON_SELECTED_GOOD_BG, FIELD_BG, FIELD_BG_FOCUSED,
    RADIUS_BUTTON, RADIUS_PANEL,
};

/// A stock deck as the picker lists and searches it.
pub(crate) struct DeckEntry {
    pub(crate) name: &'static str,
    /// The commander's full name, "A + B" for a pair.
    pub(crate) commanders: String,
    /// Lowercased name and commanders — where a search term must appear.
    key: String,
}

/// Every stock deck in `target_decks` order, so a [`DeckChoice::Stock`]
/// index is a position here. Built once: naming a commander builds its card.
pub(crate) fn deck_index() -> &'static [DeckEntry] {
    static INDEX: OnceLock<Vec<DeckEntry>> = OnceLock::new();
    INDEX.get_or_init(|| {
        crabomination::pod::target_decks()
            .iter()
            .map(|d| {
                let commanders = d.commanders.iter().map(|f| f().name).collect::<Vec<_>>().join(" + ");
                let key = format!("{} {commanders}", d.name).to_lowercase();
                DeckEntry { name: d.name, commanders, key }
            })
            .collect()
    })
}

/// Does `entry` match a search? Every whitespace-separated term has to
/// appear in its name or its commanders' names, in any case — "doctor rose"
/// finds The Tenth Doctor + Rose Tyler, "(gw)" the Selesnya decks.
pub(crate) fn deck_matches(entry: &DeckEntry, query: &str) -> bool {
    query.split_whitespace().all(|t| entry.key.contains(&t.to_lowercase()))
}

/// The picker's state: open or not, the seat a click assigns, the search.
#[derive(Resource, Default)]
pub(crate) struct DeckPicker {
    pub(crate) open: bool,
    /// 0 is the human, 1.. the bots.
    seat: usize,
    query: String,
}

#[derive(Component)]
pub(crate) struct DeckPickerRoot;

#[derive(Component)]
pub(crate) struct SeatTab(usize);

#[derive(Component)]
pub(crate) struct SeatTabText(usize);

#[derive(Component)]
pub(crate) struct DeckRow(DeckChoice);

#[derive(Component)]
pub(crate) struct DeckList;

#[derive(Component)]
pub(crate) struct SearchText;

#[derive(Component)]
pub(crate) struct DoneButton;

/// Longest search the box takes.
const MAX_QUERY: usize = 40;

/// "Decks: …" opens the picker on your own seat with an empty search.
pub(crate) fn open_deck_picker(
    q: Query<&Interaction, (Changed<Interaction>, With<PodDecksButton>)>,
    mut picker: ResMut<DeckPicker>,
    mut fields: ResMut<MenuFields>,
) {
    if q.iter().any(|i| *i == Interaction::Pressed) {
        *picker = DeckPicker { open: true, ..default() };
        // Typing goes to the search box now, not a menu field behind it.
        fields.blur();
    }
}

/// Leaving the menu closes the picker (its overlay goes with the menu).
pub(crate) fn close_deck_picker(mut picker: ResMut<DeckPicker>) {
    picker.open = false;
}

/// Spawn the overlay when the picker opens and drop it when it closes;
/// rebuild the list whenever the search changes.
pub(crate) fn sync_deck_picker(
    mut commands: Commands,
    picker: Res<DeckPicker>,
    fields: Res<MenuFields>,
    ui_fonts: Res<UiFonts>,
    roots: Query<Entity, With<DeckPickerRoot>>,
    lists: Query<Entity, With<DeckList>>,
    mut listed: Local<Option<String>>,
) {
    if !picker.open {
        for e in &roots {
            commands.entity(e).despawn();
        }
        *listed = None;
        return;
    }
    let tf = |size: f32| ui_fonts.tf(size);
    if roots.is_empty() {
        spawn_overlay(&mut commands, &tf, fields.pod_size);
        *listed = None;
        return; // the list container exists from next frame
    }
    if listed.as_deref() == Some(picker.query.as_str()) {
        return;
    }
    let Ok(list) = lists.single() else { return };
    *listed = Some(picker.query.clone());
    commands.entity(list).despawn_children();
    commands.entity(list).with_children(|l| {
        deck_row(l, &tf, DeckChoice::Random, "Random", "a different deck each game");
        for (i, d) in deck_index().iter().enumerate().filter(|(_, d)| deck_matches(d, &picker.query)) {
            deck_row(l, &tf, DeckChoice::Stock(i), d.name, &d.commanders);
        }
    });
}

fn spawn_overlay(commands: &mut Commands, tf: &impl Fn(f32) -> TextFont, pod_size: usize) {
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
            DeckPickerRoot,
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
                p.spawn((Text::new("Commander decks"), tf(22.0), TextColor(theme::ACCENT_GOLD)));
                // One tab per seat of the pod; the selected one is what a
                // click on a deck assigns.
                p.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(6.0), row_gap: Val::Px(6.0), ..default() })
                    .with_children(|tabs| {
                        for seat in 0..pod_size.clamp(2, 4) {
                            tabs.spawn((
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                                    border_radius: BorderRadius::all(RADIUS_BUTTON),
                                    ..default()
                                },
                                BackgroundColor(FIELD_BG),
                                HoverTint::new(FIELD_BG),
                                SeatTab(seat),
                                children![(Text::new(""), tf(13.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE, SeatTabText(seat))],
                            ));
                        }
                    });
                p.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        border_radius: BorderRadius::all(RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(FIELD_BG_FOCUSED),
                    children![(Text::new("Search: _"), tf(14.0), TextColor(theme::TEXT_PRIMARY), SearchText)],
                ));
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
                    DeckList,
                ));
                p.spawn(Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() })
                    .with_children(|row| {
                        row.spawn((
                            Text::new("Type to search · click a deck for the selected seat · Esc closes"),
                            tf(11.0),
                            TextColor(theme::TEXT_MUTED),
                        ));
                        row.spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)),
                                border_radius: BorderRadius::all(RADIUS_BUTTON),
                                ..default()
                            },
                            BackgroundColor(BUTTON_INFO_BG),
                            HoverTint::new(BUTTON_INFO_BG),
                            DoneButton,
                            children![(Text::new("Done"), tf(15.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE)],
                        ));
                    });
            });
        });
}

fn deck_row(l: &mut ChildSpawnerCommands, tf: &impl Fn(f32) -> TextFont, choice: DeckChoice, name: &str, detail: &str) {
    l.spawn((
        Button,
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::Px(12.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
            border_radius: BorderRadius::all(RADIUS_BUTTON),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(FIELD_BG),
        HoverTint::new(FIELD_BG),
        DeckRow(choice),
        children![
            (Text::new(name), tf(13.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE),
            (Text::new(detail), tf(11.0), TextColor(theme::TEXT_MUTED), Pickable::IGNORE),
        ],
    ));
}

/// Keep the tabs' labels and highlight, the rows' highlight (the selected
/// seat's deck) and the search text current. Writes only on a difference.
#[allow(clippy::type_complexity)]
pub(crate) fn refresh_deck_picker(
    picker: Res<DeckPicker>,
    fields: Res<MenuFields>,
    mut tab_texts: Query<(&SeatTabText, &mut Text), Without<SearchText>>,
    mut buttons: Query<(&Interaction, Option<&SeatTab>, Option<&DeckRow>, &mut HoverTint, &mut BackgroundColor)>,
    mut search: Query<&mut Text, With<SearchText>>,
) {
    if !picker.open {
        return;
    }
    for (tab, mut text) in &mut tab_texts {
        let who = if tab.0 == 0 { "You".to_string() } else { format!("Bot {}", tab.0) };
        let want = format!("{who}: {}", fields.pod_decks.seat(tab.0).label());
        if text.0 != want {
            text.0 = want;
        }
    }
    let chosen = fields.pod_decks.seat(picker.seat);
    for (interaction, tab, row, mut tint, mut bg) in &mut buttons {
        let on = match (tab, row) {
            (Some(t), _) => t.0 == picker.seat,
            (_, Some(r)) => r.0 == chosen,
            _ => continue,
        };
        let idle = match (on, tab.is_some()) {
            (false, _) => FIELD_BG,
            (true, true) => FIELD_BG_FOCUSED,
            (true, false) => BUTTON_SELECTED_GOOD_BG,
        };
        if tint.idle != idle {
            *tint = HoverTint::new(idle);
            if *interaction == Interaction::None {
                *bg = BackgroundColor(idle);
            }
        }
    }
    let want = format!("Search: {}_", picker.query);
    for mut text in &mut search {
        if text.0 != want {
            text.0 = want.clone();
        }
    }
}

/// Clicks: a tab selects its seat, a row gives the selected seat its deck,
/// Done closes.
pub(crate) fn handle_deck_picker_clicks(
    tabs: Query<(&Interaction, &SeatTab), Changed<Interaction>>,
    rows: Query<(&Interaction, &DeckRow), Changed<Interaction>>,
    done: Query<&Interaction, (Changed<Interaction>, With<DoneButton>)>,
    mut picker: ResMut<DeckPicker>,
    mut fields: ResMut<MenuFields>,
) {
    if !picker.open {
        return;
    }
    for (i, tab) in &tabs {
        if *i == Interaction::Pressed {
            picker.seat = tab.0;
        }
    }
    for (i, row) in &rows {
        if *i == Interaction::Pressed {
            let seat = picker.seat;
            fields.pod_decks.set_seat(seat, row.0);
        }
    }
    if done.iter().any(|i| *i == Interaction::Pressed) {
        picker.open = false;
    }
}

/// Typing edits the search; Backspace deletes; Esc or Enter closes.
pub(crate) fn deck_picker_keys(mut picker: ResMut<DeckPicker>, mut events: MessageReader<KeyboardInput>) {
    if !picker.open {
        events.clear();
        return;
    }
    for ev in events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match &ev.logical_key {
            Key::Backspace => {
                picker.query.pop();
            }
            Key::Escape | Key::Enter => picker.open = false,
            Key::Space if picker.query.len() < MAX_QUERY => picker.query.push(' '),
            Key::Character(s) => {
                for ch in s.chars().filter(|c| !c.is_control()) {
                    if picker.query.len() < MAX_QUERY {
                        picker.query.push(ch);
                    }
                }
            }
            _ => {}
        }
    }
}

/// The menu button's label: your deck, then the bots' in brief —
/// "Decks: Sigarda (GW) vs random", "… vs 2 picked, 1 random".
pub(crate) fn decks_summary(decks: &crate::menu::PodDecks, pod_size: usize) -> String {
    let bots = &decks.bots[..pod_size.clamp(2, 4) - 1];
    let picked = bots.iter().filter(|c| **c != DeckChoice::Random).count();
    let rest = match (picked, bots.len() - picked) {
        (0, _) => "random".to_string(),
        (k, 0) => format!("{k} picked"),
        (k, r) => format!("{k} picked, {r} random"),
    };
    format!("Decks: {} vs {rest}", decks.you.label())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menu_label_sums_up_the_bot_picks() {
        let mut decks = crate::menu::PodDecks::default();
        assert_eq!(decks_summary(&decks, 4), "Decks: Sigarda (GW) vs random");
        decks.bots[0] = DeckChoice::Stock(1);
        assert_eq!(decks_summary(&decks, 4), "Decks: Sigarda (GW) vs 1 picked, 2 random");
        assert_eq!(decks_summary(&decks, 2), "Decks: Sigarda (GW) vs 1 picked");
        decks.you = DeckChoice::Random;
        assert_eq!(decks_summary(&decks, 2), "Decks: Random vs 1 picked");
    }

    /// Every stock deck is listed, by the name the menu shows, and a search
    /// narrows by deck or commander name in any case and word order.
    #[test]
    fn the_index_lists_every_deck_and_search_narrows_it() {
        let index = deck_index();
        assert_eq!(index.len(), crabomination::pod::target_decks().len());
        assert_eq!(index[0].name, "Sigarda (GW)");
        assert_eq!(index[0].commanders, "Sigarda, Host of Herons");
        let hits = |q: &str| index.iter().filter(|d| deck_matches(d, q)).map(|d| d.name).collect::<Vec<_>>();
        assert_eq!(hits("").len(), index.len(), "no search lists everything");
        assert_eq!(hits("ROSE doctor"), ["The Tenth Doctor + Rose Tyler (URW)"]);
        assert!(hits("host of herons").contains(&"Sigarda (GW)"), "a commander's full name finds its deck");
        assert!(hits("zzzz").is_empty());
    }
}
