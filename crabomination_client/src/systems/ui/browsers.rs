//! The zone browsers: the graveyard (`G`, or a click on any pile) and exile
//! (`V`) popups, the pile tooltip, and recasting from the graveyard browser.
//! Split out of `ui.rs` (2,500 lines).

use super::*;

pub(super) const BROWSER_CARD_WIDTH: f32 = 220.0;
pub(super) const BROWSER_CARD_HEIGHT: f32 = BROWSER_CARD_WIDTH * CARD_ASPECT_RATIO;
pub(super) const BROWSER_COLS: u32 = 4;

pub fn graveyard_browser(
    mut commands: Commands,
    mut state: ResMut<GraveyardBrowserState>,
    view: Res<CurrentView>,
    asset_server: Res<AssetServer>,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<GraveyardBrowser>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    binds: Res<crate::keybinds::Keybinds>,
    overlay_interaction: Query<&Interaction, (With<GraveyardBrowser>, With<Button>)>,
    text_input: crate::systems::input_guard::TextInputGuard,
    esc: Res<crate::systems::esc::EscFocus>,
) {
    if esc.owns(crate::systems::esc::EscSurface::GraveyardBrowser) {
        state.open = false;
    }
    // `G` toggles the viewer's own graveyard (clicking a pile still browses
    // any player's). Suppressed while any text surface has the keyboard and
    // while a decision modal is up — ChooseColor binds G to green mana.
    if binds.just_pressed(&keyboard, crate::keybinds::Action::Graveyard)
        && !text_input.typing()
        && view
            .0
            .as_ref()
            .is_some_and(|cv| cv.pending_decision.is_none())
    {
        let your_seat = view.0.as_ref().map(|cv| cv.your_seat).unwrap_or(0);
        if state.open && state.owner == your_seat {
            state.open = false;
        } else {
            state.open = true;
            state.owner = your_seat;
        }
    }

    // Close when the dim overlay (outside the panel) is clicked.
    for interaction in &overlay_interaction {
        if *interaction == Interaction::Pressed {
            state.open = false;
        }
    }

    let should_show = state.open;

    if should_show && existing.is_empty() {
        let owner = state.owner;
        let owner_name: String = view
            .0
            .as_ref()
            .and_then(|cv| cv.players.iter().find(|p| p.seat == owner))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Player {owner}"));
        let owner_label = format!("{owner_name}'s");

        // (name, recast-badge, click-to-cast): badge is "Flashback {N}" /
        // "Retrace" for cards castable from the graveyard; when the browser
        // shows the viewer's own graveyard the tile also carries the recast
        // action so clicking it casts the card.
        let own_graveyard = view.0.as_ref().is_some_and(|cv| cv.your_seat == owner);
        let card_names: Vec<(String, Option<String>, Option<(crabomination::card::CardId, GraveyardRecast)>, bool)> =
            view.0.as_ref()
            // Newest first: the engine's last card is the graveyard's top.
            .map(|cv| cv.players[owner].graveyard.iter().rev().map(|c| {
                let (badge, recast) = if let Some(fb) = &c.flashback_cost {
                    (Some(format!("Flashback {{{}}}", fb.cmc())), Some(GraveyardRecast::Flashback))
                } else if let Some(mh) = &c.mayhem_cost {
                    (Some(format!("Mayhem {{{}}}", mh.cmc())), Some(GraveyardRecast::Mayhem))
                } else if let Some(hm) = &c.harmonize_cost {
                    (Some(format!("Harmonize {{{}}}", hm.cmc())), Some(GraveyardRecast::Harmonize))
                } else if let Some(db) = &c.disturb_cost {
                    (Some(format!("Disturb {{{}}}", db.cmc())), Some(GraveyardRecast::Disturb))
                } else if let Some((esc, n)) = &c.escape {
                    (Some(format!("Escape {{{}}} +{n} exile", esc.cmc())), Some(GraveyardRecast::Escape(*n)))
                } else if c.retrace {
                    (Some("Retrace".to_string()), Some(GraveyardRecast::Retrace))
                } else {
                    (None, None)
                };
                (c.name.clone(), badge, recast.filter(|_| own_graveyard).map(|r| (c.id, r)), c.needs_target)
            }).collect())
            .unwrap_or_default();
        let count = card_names.len();

        let panel_width = BROWSER_CARD_WIDTH * BROWSER_COLS as f32 + 80.0;

        let root = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(theme::OVERLAY_BG),
                Button,
                GraveyardBrowser,
                GlobalZIndex(theme::layer::MODAL),
            ))
            .id();

        let panel = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    row_gap: Val::Px(12.0),
                    width: Val::Px(panel_width),
                    max_height: Val::Percent(85.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
                bevy::ui_widgets::ScrollArea,
                crate::systems::ui_card_hover::PreviewAnchor,
            ))
            .id();

        commands.entity(root).add_child(panel);

        commands.entity(panel)
            .with_children(|panel| {
                // Header row
                panel
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Text::new(format!("{owner_label} Graveyard ({count} cards) · newest first")),
                            ui_fonts.tf(18.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Pickable::IGNORE,
                        ));
                    });

                // Card grid
                if card_names.is_empty() {
                    panel.spawn((
                        Text::new("Empty"),
                        ui_fonts.tf(14.0),
                        TextColor(theme::TEXT_SECONDARY),
                        Pickable::IGNORE,
                    ));
                } else {
                    panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(8.0),
                                row_gap: Val::Px(8.0),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|grid| {
                            for (name, badge, recast, needs_target) in &card_names {
                                let path = scryfall::card_asset_path(name);
                                let texture: Handle<Image> = asset_server.load(&path);
                                // Each tile is a Button so Bevy's
                                // `Interaction` component reports
                                // hover/press. The ImageNode inside
                                // is `Pickable::IGNORE` so events
                                // bubble up to the button rather than
                                // landing on the image directly. The
                                // outer button captures clicks too,
                                // which prevents accidental close-on-
                                // click since the dim overlay used to
                                // swallow clicks that passed through
                                // the cards.
                                grid.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(BROWSER_CARD_WIDTH),
                                        height: Val::Px(BROWSER_CARD_HEIGHT),
                                        ..default()
                                    },
                                    GraveyardCardItem {
                                        name: name.clone(),
                                        recast: *recast,
                                        needs_target: *needs_target,
                                    },
                                    // Rules text beside the tile, like the table's.
                                    crate::systems::ui_card_hover::UiCardHover::card(name, recast.map(|(id, _)| id)),
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
                                    // Recast badge pinned to the bottom of
                                    // the tile (Flashback / Retrace).
                                    if let Some(label) = badge {
                                        tile.spawn((
                                            Node {
                                                position_type: PositionType::Absolute,
                                                bottom: Val::Px(2.0),
                                                left: Val::Px(2.0),
                                                right: Val::Px(2.0),
                                                justify_content: JustifyContent::Center,
                                                ..default()
                                            },
                                            BackgroundColor(theme::OVERLAY_BG),
                                            Pickable::IGNORE,
                                        ))
                                        .with_children(|b| {
                                            b.spawn((
                                                Text::new(label.clone()),
                                                ui_fonts.tf(10.0),
                                                TextColor(theme::TEXT_PRIMARY),
                                                Pickable::IGNORE,
                                            ));
                                        });
                                    }
                                });
                            }
                        });
                }

                // Close hint
                panel.spawn((
                    Text::new("Press Esc or click outside to close"),
                    ui_fonts.tf(11.0),
                    TextColor(theme::TEXT_MUTED),
                    Pickable::IGNORE,
                ));
            });
    } else if !should_show {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
    }
}

/// Exile-zone browser: V toggles a popup listing every exiled card with
/// badges for the engine state that matters (face down, may-play grants,
/// Cipher encodes, linked "until X leaves" exiles). Mirrors the graveyard
/// browser's layout; tiles reuse `GraveyardCardItem` so the shared
/// hover-name tooltip works.
pub fn exile_browser(
    mut commands: Commands,
    view: Res<CurrentView>,
    asset_server: Res<AssetServer>,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<ExileBrowser>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    binds: Res<crate::keybinds::Keybinds>,
    overlay_interaction: Query<&Interaction, (With<ExileBrowser>, With<Button>)>,
    text_input: crate::systems::input_guard::TextInputGuard,
    esc: Res<crate::systems::esc::EscFocus>,
    mut state: ResMut<crate::game::ExileBrowserState>,
    card_names: Res<crate::game::CardNames>,
) {
    // Two ways in: the `V` key, and clicking the shared exile pile (which
    // flips `state.open`). The pile is the discoverable one — a browser
    // reachable only by knowing a keybind may as well not exist.
    let key_v = !text_input.typing() && binds.just_pressed(&keyboard, crate::keybinds::Action::Exile);
    if key_v {
        state.open = !state.open;
        state.owner = None;
    }
    // Clicking the scrim closes without consuming the Esc press.
    let close_requested = overlay_interaction.iter().any(|i| *i == Interaction::Pressed)
        || esc.owns(crate::systems::esc::EscSurface::ExileBrowser);
    if !existing.is_empty() && (close_requested || !state.open) {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
        state.open = false;
        state.owner = None;
        return;
    }
    if !existing.is_empty() || !state.open {
        return;
    }
    let Some(cv) = view.0.as_ref() else { return };
    // An opponent panel's exile count opens the browser on that seat's
    // cards only; the `V` key and the shared pile show everything.
    let owner_filter = state.owner;

    // (owner, display name, badge, face_down, may-play id) per exiled card.
    // The last slot is `Some` only when the engine says the viewer could
    // play it *right now* (`may_play_castable` is the same `would_accept`
    // dry-run as every other affordance), so the tile is clickable exactly
    // when the click would be accepted.
    let entries: Vec<(
        usize,
        String,
        Option<String>,
        bool,
        Option<(crabomination::card::CardId, GraveyardRecast)>,
    )> = cv
        .exile
        .iter()
        .filter(|c| owner_filter.is_none_or(|o| c.owner == o))
        .map(|c| {
            let mut badges: Vec<String> = Vec::new();
            if c.face_down {
                badges.push("Face down".to_string());
            }
            if let Some(p) = c.may_play_recipient {
                let who = if p == cv.your_seat { "you".to_string() } else { format!("P{p}") };
                // Surface airbend's flat {2} / miracle / Hostage-Taker alt cost.
                badges.push(match c.may_play_alt_cost {
                    Some(n) => format!("May play ({who}) for {{{n}}}"),
                    None => format!("May play ({who})"),
                });
            }
            if let Some(life) = c.play_for_life {
                badges.push(format!("Play for {life} life"));
            }
            if c.plotted {
                // CR 702.170d — free sorcery-speed cast on a later turn.
                badges.push(if cv.castable_plotted.contains(&c.id) {
                    "Plotted — castable now".to_string()
                } else {
                    "Plotted".to_string()
                });
            }
            if c.on_adventure {
                // CR 715.3d — a land half is played, everything else is cast.
                badges.push(if cv.adventure_exile.contains(&c.id) {
                    "On an adventure — playable now".to_string()
                } else {
                    "On an adventure".to_string()
                });
            }
            if c.encoded_on.is_some() {
                badges.push("Cipher: encoded".to_string());
            }
            if let Some(by) = c.exiled_by {
                // Name the exiler where we can — "why is this card here"
                // is the question the browser exists to answer, and a bare
                // "returns when exiler leaves" doesn't answer it.
                match card_names.by_id.get(&by) {
                    Some(name) => badges.push(format!("Exiled by {name} — returns when it leaves")),
                    None => badges.push("Returns when the exiler leaves".to_string()),
                }
            }
            // One permission, two actions: a nonland is cast
            // (`CastFromZoneWithoutPaying`), a land is played
            // (`PlayLand`). The engine publishes them as separate lists
            // because the click has to send a different action — before
            // `may_play_lands` existed only the cast list was consulted,
            // so a may-play land showed its badge and never became
            // clickable.
            let play = if cv.may_play_castable.contains(&c.id) {
                Some((c.id, GraveyardRecast::MayPlay))
            } else if cv.may_play_lands.contains(&c.id) {
                Some((c.id, GraveyardRecast::MayPlayLand))
            } else {
                None
            };
            if play.is_some() {
                badges.push("Click to play".to_string());
            }
            let badge = (!badges.is_empty()).then(|| badges.join(" · "));
            (c.owner, c.name.clone(), badge, c.face_down, play)
        })
        .collect();
    // Owner sections: the viewer first, then the remaining seats in order —
    // mirrors the graveyard browser's per-owner split instead of one
    // merged pile.
    let mut owners: Vec<usize> = entries.iter().map(|(o, ..)| *o).collect();
    owners.sort_unstable();
    owners.dedup();
    owners.sort_by_key(|&o| (o != cv.your_seat, o));
    let owner_label = |seat: usize| -> String {
        if seat == cv.your_seat {
            "You".to_string()
        } else {
            cv.players
                .iter()
                .find(|p| p.seat == seat)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| format!("P{seat}"))
        }
    };
    let count = entries.len();
    let panel_width = BROWSER_CARD_WIDTH * BROWSER_COLS as f32 + 80.0;

    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG),
            Button,
            ExileBrowser,
            GlobalZIndex(theme::layer::MODAL),
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(12.0),
                width: Val::Px(panel_width),
                max_height: Val::Percent(85.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
            bevy::ui_widgets::ScrollArea,
            crate::systems::ui_card_hover::PreviewAnchor,
        ))
        .id();
    commands.entity(root).add_child(panel);

    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(match owner_filter {
                Some(o) => format!("Exile — {}'s cards ({count})", owner_label(o)),
                None => format!("Exile ({count} cards)"),
            }),
            ui_fonts.tf(18.0),
            TextColor(theme::TEXT_PRIMARY),
            Pickable::IGNORE,
        ));
        if entries.is_empty() {
            panel.spawn((
                Text::new("Empty"),
                ui_fonts.tf(14.0),
                TextColor(theme::TEXT_SECONDARY),
                Pickable::IGNORE,
            ));
        } else {
            for owner in owners {
                let section: Vec<_> =
                    entries.iter().filter(|(o, ..)| *o == owner).collect();
                panel.spawn((
                    Text::new(format!("{} ({})", owner_label(owner), section.len())),
                    ui_fonts.tf(14.0),
                    TextColor(theme::TEXT_BODY),
                    Pickable::IGNORE,
                ));
                panel
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(8.0),
                            row_gap: Val::Px(8.0),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|grid| {
                        for (_, name, badge, face_down, may_play) in section {
                        let tile_children = |tile: &mut ChildSpawnerCommands| {
                            if !face_down {
                                let texture: Handle<Image> =
                                    asset_server.load(scryfall::card_asset_path(name));
                                tile.spawn((
                                    ImageNode { image: texture, ..default() },
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    Pickable::IGNORE,
                                ));
                            } else {
                                // Masked card: a blank tile with the name.
                                tile.spawn((
                                    Text::new(name.clone()),
                                    ui_fonts.tf(12.0),
                                    TextColor(theme::TEXT_SECONDARY),
                                    Pickable::IGNORE,
                                ));
                            }
                            if let Some(label) = badge {
                                tile.spawn((
                                    Node {
                                        position_type: PositionType::Absolute,
                                        bottom: Val::Px(2.0),
                                        left: Val::Px(2.0),
                                        right: Val::Px(2.0),
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BackgroundColor(theme::OVERLAY_BG),
                                    Pickable::IGNORE,
                                ))
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new(label.clone()),
                                        ui_fonts.tf(10.0),
                                        TextColor(theme::TEXT_PRIMARY),
                                        Pickable::IGNORE,
                                    ));
                                });
                            }
                        };
                        grid.spawn((
                            Button,
                            Node {
                                width: Val::Px(BROWSER_CARD_WIDTH),
                                height: Val::Px(BROWSER_CARD_HEIGHT),
                                ..default()
                            },
                            GraveyardCardItem {
                                name: name.clone(),
                                recast: *may_play,
                                needs_target: false,
                            },
                            crate::systems::ui_card_hover::UiCardHover::card(name, may_play.map(|(id, _)| id)),
                        ))
                        .with_children(tile_children);
                    }
                });
            }
        }
        // CR 717.4 — the Attraction junkyard is its own out-of-play pile.
        // It rides the exile browser rather than a fourth overlay: both are
        // "cards that left play and aren't coming back on their own".
        for p in &cv.players {
            if p.attraction_junkyard.is_empty() {
                continue;
            }
            panel.spawn((
                Text::new(format!(
                    "{} — junkyard ({})",
                    owner_label(p.seat),
                    p.attraction_junkyard.len()
                )),
                ui_fonts.tf(14.0),
                TextColor(theme::TEXT_BODY),
                Pickable::IGNORE,
            ));
            panel
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(8.0),
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|grid| {
                    for name in p.attraction_junkyard.iter() {
                        let texture: Handle<Image> =
                            asset_server.load(scryfall::card_asset_path(name));
                        grid.spawn((
                            Button,
                            Node {
                                width: Val::Px(BROWSER_CARD_WIDTH),
                                height: Val::Px(BROWSER_CARD_HEIGHT),
                                ..default()
                            },
                            GraveyardCardItem { name: name.clone(), recast: None, needs_target: false },
                            crate::systems::ui_card_hover::UiCardHover::card(name, None),
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
                });
        }
        panel.spawn((
            Text::new("Press V or Esc, or click outside to close"),
            ui_fonts.tf(11.0),
            TextColor(theme::TEXT_MUTED),
            Pickable::IGNORE,
        ));
    });
}

pub fn pile_tooltip(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    pile_hovered: Query<&DeckPile, With<PileHovered>>,
    gy_hovered: Query<&GraveyardPile, With<PileHovered>>,
    existing: Query<(Entity, &PileTooltip)>,
) {
    let cv = view.0.as_ref();
    let lib_size = |owner: usize| -> usize {
        cv.and_then(|cv| cv.players.iter().find(|p| p.seat == owner))
            .map(|p| p.library.size)
            .unwrap_or(0)
    };
    let gy_count = |owner: usize| -> usize {
        cv.and_then(|cv| cv.players.iter().find(|p| p.seat == owner))
            .map(|p| p.graveyard.len())
            .unwrap_or(0)
    };
    let player_name = |owner: usize| -> String {
        cv.and_then(|cv| cv.players.iter().find(|p| p.seat == owner))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Player {owner}"))
    };
    let text = if let Some(pile) = pile_hovered.iter().next() {
        let count = lib_size(pile.owner);
        Some(format!("{} library: {count} card{}", player_name(pile.owner), if count == 1 { "" } else { "s" }))
    } else if let Some(gy) = gy_hovered.iter().next() {
        let count = gy_count(gy.owner);
        Some(format!("{}'s graveyard: {count} card{} — click to browse", player_name(gy.owner), if count == 1 { "" } else { "s" }))
    } else {
        None
    };

    if let Some(msg) = text {
        // If a tooltip is already pinned with the *same* text, leave it.
        // Otherwise despawn the stale one so we can respawn with the
        // current message (fixes the cursor-moved-between-piles staleness).
        let up_to_date = existing.iter().any(|(_, t)| t.msg == msg);
        if !up_to_date {
            for (e, _) in existing.iter() {
                commands.entity(e).despawn();
            }
            // Pin the tooltip's *center* to the screen midline. Without
            // the negative margin Bevy positions the node's left edge at
            // 50% and the tooltip drifts right of center (worse the
            // longer the string). 140px is half the widest tooltip text
            // ("Player N's graveyard: NN cards — click to browse") at
            // the 14px font, which is close enough for visual centering
            // without measuring text width per frame.
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(130.0),
                    left: Val::Percent(50.0),
                    margin: UiRect::left(Val::Px(-140.0)),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::OVERLAY_BG_HEAVY),
                PileTooltip { msg: msg.clone() },
                GlobalZIndex(theme::layer::CARD_PEEK),
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new(msg),
                    ui_fonts.tf(14.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
        }
    } else {
        for (entity, _) in &existing {
            commands.entity(entity).despawn();
        }
    }
}

/// Click-to-cast for the viewer's own castable graveyard tiles, left or
/// right button (right-click is the table's "do the useful thing" gesture):
/// submits the recast action carried by the tile (Escape auto-picks its
/// exile fodder) and closes the browser. A flashback whose spell needs a
/// target arms the targeting cursor instead, so the pick is the player's —
/// the same flow a targeted hand cast takes.
pub fn graveyard_recast_click(
    items: Query<(Ref<Interaction>, &GraveyardCardItem)>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut targeting: ResMut<crate::game::TargetingState>,
    mut state: ResMut<GraveyardBrowserState>,
    mut exile_state: ResMut<crate::game::ExileBrowserState>,
    view: Res<CurrentView>,
    outbox: Option<Res<crate::net_plugin::NetOutbox>>,
) {
    let Some(outbox) = outbox else { return };
    for (interaction, item) in &items {
        let left = *interaction == Interaction::Pressed && interaction.is_changed();
        let right = mouse.just_pressed(MouseButton::Right) && *interaction != Interaction::None;
        if !(left || right) {
            continue;
        }
        let Some((card_id, recast)) = &item.recast else { continue };
        if matches!(recast, GraveyardRecast::Flashback) && item.needs_target {
            targeting.active = true;
            targeting.pending_card_id = Some(*card_id);
            targeting.back_face_pending = false;
            targeting.pending_cast_variant = Some(crate::game::HandCastVariant::Flashback);
            state.open = false;
            exile_state.open = false;
            return;
        }
        use crabomination::game::GameAction;
        let action = match recast {
            GraveyardRecast::Flashback => GameAction::CastFlashback {
                card_id: *card_id, target: None, additional_targets: vec![],
                mode: None, x_value: None,
            },
            GraveyardRecast::Mayhem => GameAction::CastMayhem {
                card_id: *card_id, target: None, additional_targets: vec![],
                mode: None, x_value: None,
            },
            GraveyardRecast::Harmonize => GameAction::CastHarmonize {
                card_id: *card_id, tap_creature: None, target: None,
                additional_targets: vec![], mode: None, x_value: None,
            },
            GraveyardRecast::Disturb => GameAction::CastDisturb {
                card_id: *card_id, target: None, additional_targets: vec![],
            },
            GraveyardRecast::Retrace => GameAction::CastRetrace {
                card_id: *card_id, target: None, additional_targets: vec![],
                mode: None, x_value: None,
            },
            GraveyardRecast::MayPlay => GameAction::CastFromZoneWithoutPaying {
                card_id: *card_id, target: None, additional_targets: vec![],
                mode: None, x_value: None,
            },
            // A land under the same permission is played, not cast.
            GraveyardRecast::MayPlayLand => GameAction::PlayLand(*card_id),
            GraveyardRecast::Escape(n) => {
                // Auto-pick the first N *other* cards of the viewer's
                // graveyard as the exile payment.
                let exile_cards: Vec<crabomination::card::CardId> = view.0.as_ref()
                    .map(|cv| {
                        cv.players[state.owner].graveyard.iter()
                            .filter(|c| c.id != *card_id)
                            .take(*n as usize)
                            .map(|c| c.id)
                            .collect()
                    })
                    .unwrap_or_default();
                if exile_cards.len() < *n as usize {
                    continue; // not enough fodder — leave the browser open
                }
                GameAction::CastEscape {
                    card_id: *card_id, exile_cards, target: None,
                    additional_targets: vec![], mode: None, x_value: None,
                }
            }
        };
        outbox.submit(action);
        // Tiles are shared between the two browsers, so close both — the
        // click came from whichever one is open.
        state.open = false;
        exile_state.open = false;
    }
}

/// Show the name of the graveyard card currently being hovered as
/// a tooltip pinned to the top of the browser panel. Despawns when
/// the cursor leaves all card tiles, or when the browser closes.
pub fn graveyard_card_hover_name(
    mut commands: Commands,
    ui_fonts: Res<UiFonts>,
    items: Query<(&Interaction, &GraveyardCardItem)>,
    existing: Query<Entity, With<GraveyardCardNameTooltip>>,
    browser: Query<(), Or<(With<GraveyardBrowser>, With<ExileBrowser>)>>,
    mut current_text: Query<&mut Text, With<GraveyardCardNameTooltip>>,
) {
    // No browser open → drop the tooltip.
    if browser.is_empty() {
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    }

    let hovered_name = items
        .iter()
        .find(|(i, _)| matches!(**i, Interaction::Hovered | Interaction::Pressed))
        .map(|(_, item)| item.name.clone());

    let Some(name) = hovered_name else {
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    };

    // Tooltip already exists — just refresh its text.
    if let Ok(mut t) = current_text.single_mut() {
        if t.0 != name {
            t.0 = name;
        }
        return;
    }

    // Spawn the tooltip pinned near the top of the screen so it's
    // always visible even when scrolling deep into a long graveyard.
    // Marker lives on the Text entity (parent + child) — keeping it
    // single-entity makes both the despawn and the text-refresh
    // queries straightforward.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-110.0)),
                width: Val::Px(220.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG_HEAVY),
            Pickable::IGNORE,
            Text::new(name),
            ui_fonts.tf(16.0),
            TextColor(theme::ACCENT_GOLD),
            GraveyardCardNameTooltip,
        ));
}
