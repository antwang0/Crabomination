//! The draft's last two screens: deckbuilding (main / sideboard, basics)
//! and opponent select. Split out of `draft.rs` (2,500 lines), which keeps
//! the session, the pack screen and the click handlers.

use super::*;

// ── Deckbuilding screen ─────────────────────────────────────────────────────

pub(super) fn spawn_deckbuilding_screen(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    session: &DraftSession,
) {
    let root = spawn_root(commands);
    commands
        .entity(root)
        .with_children(|root| {
            // Header.
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG_RAISED),
                Pickable::IGNORE,
            ))
            .with_children(|h| {
                h.spawn((
                    Text::new("Build Your Deck"),
                    ui_fonts.tf(22.0),
                    TextColor(theme::ACCENT_GOLD),
                    Pickable::IGNORE,
                ));
                let total = session.main_total();
                let label = format!(
                    "Main: {} ({} spells + {} basics)  ·  Sideboard: {}",
                    total,
                    session.main.len(),
                    total as usize - session.main.len(),
                    session.sideboard.len(),
                );
                h.spawn((
                    Text::new(label),
                    ui_fonts.tf(14.0),
                    TextColor(theme::TEXT_BODY),
                    Pickable::IGNORE,
                ));
                // Same sort toolbar as the drafting screen — the main pile
                // is hardest to scan exactly here, at 23+ cards.
                h.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|sorts| {
                    sorts.spawn((
                        Text::new("Sort:"),
                        ui_fonts.tf(12.0),
                        TextColor(theme::TEXT_BODY),
                        Node { margin: UiRect::right(Val::Px(4.0)), ..default() },
                        Pickable::IGNORE,
                    ));
                    for sort in PackSort::ALL {
                        spawn_sort_button(sorts, ui_fonts, sort, session.pack_sort);
                    }
                });
            });

            // Body: main + sideboard side by side.
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|body| {
                // Main panel.
                body.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        flex_basis: Val::Percent(60.0),
                        max_height: Val::Px(600.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor(theme::PANEL_BG_RAISED),
                    bevy::ui_widgets::ScrollArea,
                ))
                .with_children(|main| {
                    main.spawn((
                        Text::new("Main deck — click a card to move it to the sideboard"),
                        ui_fonts.tf(14.0),
                        TextColor(theme::TEXT_BODY),
                        Pickable::IGNORE,
                    ));
                    main.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(6.0),
                            row_gap: Val::Px(6.0),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|grid| {
                        // Display-sorted; `i` stays the original index so
                        // move-to-sideboard clicks hit the right card.
                        for &i in &sorted_pack_indices(&session.main, session.pack_sort) {
                            let factory = session.main[i];
                            spawn_deckbuild_tile(grid, asset_server, ui_fonts, factory().name, true, i);
                        }
                    });
                });

                // Sideboard + basic-land controls.
                body.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        flex_basis: Val::Percent(40.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|right| {
                    // Basic land controls.
                    right.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(theme::PANEL_BG_RAISED),
                        Pickable::IGNORE,
                    ))
                    .with_children(|panel| {
                        panel.spawn((
                            Text::new("Basic lands"),
                            ui_fonts.tf(14.0),
                            TextColor(theme::ACCENT_GOLD),
                            Pickable::IGNORE,
                        ));
                        for color in
                            [ManaColor::White, ManaColor::Blue, ManaColor::Black, ManaColor::Red, ManaColor::Green]
                        {
                            let count = session.basics.get(&color).copied().unwrap_or(0);
                            spawn_basic_row(panel, ui_fonts, color, count);
                        }
                    });

                    // Sideboard list.
                    right.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            max_height: Val::Px(400.0),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        BackgroundColor(theme::PANEL_BG_RAISED),
                        bevy::ui_widgets::ScrollArea,
                    ))
                    .with_children(|sb| {
                        sb.spawn((
                            Text::new("Sideboard — click a card to move it to the main deck"),
                            ui_fonts.tf(13.0),
                            TextColor(theme::TEXT_BODY),
                            Pickable::IGNORE,
                        ));
                        sb.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(6.0),
                                row_gap: Val::Px(6.0),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|grid| {
                            for &i in &sorted_pack_indices(&session.sideboard, session.pack_sort) {
                                let factory = session.sideboard[i];
                                spawn_deckbuild_tile(grid, asset_server, ui_fonts, factory().name, false, i);
                            }
                        });
                    });

                    // Confirm button.
                    let confirm_enabled = session.main_total() >= 40;
                    let confirm_label = if confirm_enabled {
                        format!("Confirm Deck ({} cards) →", session.main_total())
                    } else {
                        format!(
                            "Need at least 40 cards (currently {})",
                            session.main_total()
                        )
                    };
                    let confirm_bg = if confirm_enabled {
                        theme::ACCENT_GREEN
                    } else {
                        theme::BUTTON_DISABLED_BG
                    };
                    let mut confirm = right.spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(confirm_bg),
                        ConfirmDeckButton,
                    ));
                    // Only brighten on hover while the action is actually
                    // available — a disabled button shouldn't feel clickable.
                    if confirm_enabled {
                        confirm.insert(theme::HoverTint::new(confirm_bg));
                    }
                    confirm.with_children(|b| {
                        b.spawn((
                            Text::new(confirm_label),
                            ui_fonts.tf(16.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Pickable::IGNORE,
                        ));
                    });

                    // Save-deck button — writes the staged list as importable
                    // decklist text so the draft survives past this session.
                    right
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(theme::BUTTON_INFO_BG),
                            theme::HoverTint::new(theme::BUTTON_INFO_BG),
                            SaveDeckButton,
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("Save Deck"),
                                ui_fonts.tf(14.0),
                                TextColor(theme::TEXT_PRIMARY),
                                Pickable::IGNORE,
                            ));
                        });
                    if let Some(status) = &session.save_status {
                        right.spawn((
                            Text::new(status.clone()),
                            ui_fonts.tf(12.0),
                            TextColor(theme::TEXT_BODY),
                            Pickable::IGNORE,
                        ));
                    }
                });
            });
        });
}

pub(super) fn spawn_deckbuild_tile(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    _ui_fonts: &UiFonts,
    card_name: &str,
    in_main: bool,
    index: usize,
) {
    let path = scryfall::card_asset_path(card_name);
    let texture: Handle<Image> = asset_server.load(&path);
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(DECKBUILD_CARD_W),
                height: Val::Px(DECKBUILD_CARD_H),
                ..default()
            },
            BackgroundColor(theme::FIELD_BG),
            theme::HoverTint::new(theme::FIELD_BG),
            DeckbuildCardButton { in_main, index },
        ))
        .with_children(|tile| {
            tile.spawn((
                ImageNode { image: texture, ..default() },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn spawn_basic_row(parent: &mut ChildSpawnerCommands, ui_fonts: &UiFonts, color: ManaColor, count: u32) {
    let label = match color {
        ManaColor::White => "Plains",
        ManaColor::Blue => "Island",
        ManaColor::Black => "Swamp",
        ManaColor::Red => "Mountain",
        ManaColor::Green => "Forest",
    };
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(format!("{label}:")),
                ui_fonts.tf(14.0),
                TextColor(theme::TEXT_BODY),
                Node { min_width: Val::Px(70.0), ..default() },
                Pickable::IGNORE,
            ));
            // Minus button.
            row.spawn((
                Button,
                Node {
                    width: Val::Px(28.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(theme::BUTTON_DANGER_BG),
                theme::HoverTint::new(theme::BUTTON_DANGER_BG),
                BasicAdjustButton { color, delta: -1 },
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("−"),
                    ui_fonts.tf(16.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
            // Count.
            row.spawn((
                Text::new(format!("{count}")),
                ui_fonts.tf(14.0),
                TextColor(theme::TEXT_PRIMARY),
                Node { min_width: Val::Px(28.0), ..default() },
                Pickable::IGNORE,
            ));
            // Plus button.
            row.spawn((
                Button,
                Node {
                    width: Val::Px(28.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                theme::HoverTint::new(theme::BUTTON_PRIMARY_BG),
                BasicAdjustButton { color, delta: 1 },
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("+"),
                    ui_fonts.tf(16.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
        });
}

// ── Opponent select screen ──────────────────────────────────────────────────

pub(super) fn spawn_opponent_select_screen(
    commands: &mut Commands,
    _asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    session: &DraftSession,
) {
    let root = spawn_root(commands);
    commands
        .entity(root)
        .with_children(|root| {
            // Header.
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(14.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG_RAISED),
                Pickable::IGNORE,
            ))
            .with_children(|h| {
                h.spawn((
                    Text::new("Pick Your Opponent"),
                    ui_fonts.tf(24.0),
                    TextColor(theme::ACCENT_GOLD),
                    Pickable::IGNORE,
                ));
                h.spawn((
                    Text::new("Each bot drafted its own deck. Click one to play against them."),
                    ui_fonts.tf(14.0),
                    TextColor(theme::TEXT_BODY),
                    Pickable::IGNORE,
                ));
            });

            // Grid of 7 opponent buttons (seats 1..8, skipping the user).
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(16.0),
                    row_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|grid| {
                for seat in 0..POD_SIZE {
                    if seat == session.user_seat {
                        continue;
                    }
                    let colors = top_two_colors(&session.pod.picks[seat]);
                    let label = format!(
                        "Bot {seat}\n{}{}\n{} picks",
                        color_short(colors[0]),
                        color_short(colors[1]),
                        session.pod.picks[seat].len()
                    );
                    grid.spawn((
                        Button,
                        Node {
                            width: Val::Px(180.0),
                            height: Val::Px(120.0),
                            padding: UiRect::all(Val::Px(12.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(theme::FIELD_BG),
                        theme::HoverTint::new(theme::FIELD_BG),
                        OpponentChoiceButton { seat },
                    ))
                    .with_children(|tile| {
                        tile.spawn((
                            Text::new(label),
                            ui_fonts.tf(16.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Pickable::IGNORE,
                        ));
                    });
                }
            });

            // Selected-opponent confirmation row + Play button.
            if let Some(seat) = session.chosen_opponent {
                let colors = top_two_colors(&session.pod.picks[seat]);
                root.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(format!(
                            "Selected: Bot {seat} ({}{})",
                            color_short(colors[0]),
                            color_short(colors[1]),
                        )),
                        ui_fonts.tf(18.0),
                        TextColor(theme::ACCENT_GOLD),
                        Pickable::IGNORE,
                    ));
                    row.spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(28.0), Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(theme::ACCENT_GREEN),
                        PlayMatchButton,
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("Play Match →"),
                            ui_fonts.tf(18.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Pickable::IGNORE,
                        ));
                    });
                });
            }

            // Back-to-menu escape hatch.
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|row| {
                row.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                    theme::HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                    DraftBackToMenuButton,
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("Back to Menu"),
                        ui_fonts.tf(14.0),
                        TextColor(theme::TEXT_PRIMARY),
                        Pickable::IGNORE,
                    ));
                });
            });
        });
}
