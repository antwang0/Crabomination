//! The 2-D stack panel: each item on the stack, top first, with what it
//! targets and a "Let resolve" button while the viewer holds priority.

use super::*;

// ── Stack panel ───────────────────────────────────────────────────────────────

/// Resolve a `Target` to a display string given the current view.
fn target_display(cv: &crabomination::net::ClientView, tgt: &Target) -> String {
    match tgt {
        Target::Player(s) => {
            if *s == cv.your_seat { "you".into() } else { player_name(cv, *s) }
        }
        Target::Permanent(id) => cv
            .battlefield
            .iter()
            .find(|p| p.id == *id)
            .map(|p| p.name.clone())
            .or_else(|| {
                // Check graveyards (e.g. Goryo's Vengeance).
                cv.players.iter().flat_map(|p| &p.graveyard)
                    .find(|g| g.id == *id)
                    .map(|g| g.name.clone())
            })
            .unwrap_or_else(|| "target".into()),
    }
}

/// "Let resolve ▶" button in the stack panel's footer — an in-context way
/// to pass priority while reading the stack (same action as Space / the
/// Pass button).
#[derive(Component)]
pub struct StackResolveButton;

/// Submit a priority pass when the stack panel's resolve button is clicked.
pub fn handle_stack_resolve_button(
    outbox: Option<Res<NetOutbox>>,
    q: Query<&Interaction, (Changed<Interaction>, With<StackResolveButton>)>,
) {
    if q.iter().any(|i| *i == Interaction::Pressed)
        && let Some(outbox) = &outbox
    {
        outbox.submit(GameAction::PassPriority);
    }
}

/// The stack panel's outer node, placed by [`place_stack_panel`].
#[derive(Component)]
pub struct StackPanelAnchor;

/// Put the stack panel beside the 3-D stack lane (`framing::StackLane`),
/// top-aligned with it on its left, so the stack reads in one place beside
/// the table. Bottom-centre, it lay over the viewer's lands — at 1280x720,
/// over their creatures.
pub fn place_stack_panel(
    home: Res<crate::systems::camera_zoom::CameraHome>,
    ui_scale: Res<UiScale>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut anchors: Query<&mut Node, With<StackPanelAnchor>>,
) {
    if !home.is_changed() && !ui_scale.is_changed() {
        return;
    }
    let lane = home.stack_lane.screen;
    let Ok(window) = windows.single() else { return };
    if lane.is_empty() {
        return;
    }
    // Logical px → UI px.
    let s = ui_scale.0;
    for mut node in &mut anchors {
        node.top = Val::Px(lane.min.y / s);
        node.bottom = Val::Auto;
        node.left = Val::Auto;
        node.right = Val::Px((window.width() - lane.min.x) / s + 8.0);
        node.justify_content = JustifyContent::FlexEnd;
        node.align_items = AlignItems::FlexStart;
    }
}

/// Rebuild the `StackPanel` children whenever the view changes.
/// Stack is LIFO: the last element resolves next.  We show top-of-stack
/// first, as a card-art tile pile: the top item gets a large gold-framed
/// tile ("resolves next"), the rest smaller rows beneath, each with a
/// controller-colored edge strip (green = yours, orange = an opponent's).
/// A footer offers "Let resolve ▶" when the viewer holds priority.
#[allow(clippy::too_many_arguments)]
pub fn update_stack_panel(
    view: Res<CurrentView>,
    mut commands: Commands,
    mut panel_q: Query<(Entity, &mut Node), With<StackPanel>>,
    ui_fonts: Res<UiFonts>,
) {
    if !view.is_changed() {
        return;
    }
    let Ok((panel_entity, mut node)) = panel_q.single_mut() else { return };

    // Always wipe and rebuild — stack changes are infrequent.
    commands.entity(panel_entity).despawn_children();

    let Some(cv) = &view.0 else {
        node.display = Display::None;
        return;
    };

    if cv.stack.is_empty() {
        node.display = Display::None;
        return;
    }

    node.display = Display::Flex;

    let tf = |size: f32| ui_fonts.tf(size);

    let your_priority = cv.priority == cv.your_seat;
    let priority_name = if your_priority {
        "You".to_string()
    } else {
        player_name(cv, cv.priority)
    };
    let header = format!(
        "Stack  {}  |  {} has priority",
        if cv.stack.len() == 1 { "1 item".to_string() } else { format!("{} items", cv.stack.len()) },
        priority_name,
    );

    commands.entity(panel_entity).with_children(|p| {
        // Header row
        p.spawn((
            Text::new(header),
            tf(12.0),
            TextColor(theme::TEXT_BODY),
        ));
        // Divider
        p.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(2.0)),
            ..default()
        });

        // Items: top of stack (last index) shown first — that's what resolves
        // next, rendered as a large gold-framed art tile; the rest get small
        // thumbnail rows. Each row carries a controller-colored edge strip
        // (green = yours, orange = an opponent's) and triggers keep their
        // tinted badge so a surprise trigger still pops.
        use crabomination::net::{StackItemKind, StackItemView};
        for (offset, item) in cv.stack.iter().rev().enumerate() {
            let is_top = offset == 0;
            let (kind_str, kind_color, name, ctrl_seat, tgt_str, is_trigger, art_path) =
                match item {
                    StackItemView::Known(k) => {
                        let (kstr, kcol, is_trig) = match k.kind {
                            StackItemKind::Spell => ("SPELL", theme::ACCENT_ORANGE, false),
                            StackItemKind::Trigger => ("⚡ TRIGGER", theme::ACCENT_BLUE, true),
                        };
                        // Every chosen target — primary slot plus the
                        // additional-target slots of multi-target spells
                        // (Divide-damage, Support, fused splits).
                        let all_targets: Vec<String> = k
                            .target
                            .iter()
                            .chain(k.additional_targets.iter())
                            .map(|t| target_display(cv, t))
                            .collect();
                        let tgt = if all_targets.is_empty() {
                            String::new()
                        } else {
                            format!("→ {}", all_targets.join(", "))
                        };
                        (
                            kstr,
                            kcol,
                            k.name.clone(),
                            k.controller,
                            tgt,
                            is_trig,
                            crate::scryfall::card_asset_path(&k.name),
                        )
                    }
                    StackItemView::Hidden { controller, .. } => (
                        "?",
                        theme::TEXT_MUTED,
                        "Hidden card".to_string(),
                        *controller,
                        String::new(),
                        false,
                        "cardback.png".to_string(),
                    ),
                };
            let mine = ctrl_seat == cv.your_seat;
            let ctrl_str =
                if mine { "You".to_string() } else { player_name(cv, ctrl_seat) };
            let edge = if mine { theme::ACCENT_GREEN } else { theme::ACCENT_ORANGE };
            let name_color = if is_trigger { theme::ACCENT_BLUE } else { theme::TEXT_PRIMARY };
            // The rows carry no art: the 3-D stack lane beside the panel
            // shows the cards, and a row's hover opens the full one.
            let strip_h = if is_top { 34.0 } else { 26.0 };

            let row_bg = if is_top {
                Color::srgba(0.16, 0.14, 0.06, 0.85)
            } else if is_trigger {
                Color::srgba(
                    kind_color.to_srgba().red,
                    kind_color.to_srgba().green,
                    kind_color.to_srgba().blue,
                    0.10,
                )
            } else {
                Color::NONE
            };
            let mut row_ec = p.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    padding: UiRect::axes(Val::Px(4.0), Val::Px(3.0)),
                    border: UiRect::all(Val::Px(if is_top { 1.0 } else { 0.0 })),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor::all(if is_top { theme::ACCENT_GOLD } else { Color::NONE }),
                BackgroundColor(row_bg),
            ));
            // Known items show the full card on hover (hidden ones have
            // nothing to reveal).
            if let StackItemView::Known(k) = item {
                row_ec.insert((
                    Button,
                    crate::systems::ui_card_hover::UiCardHover { path: art_path.clone(), name: k.name.clone(), id: Some(k.source) },
                ));
            }
            row_ec.with_children(|row| {
                // Controller edge strip.
                row.spawn((
                    Node {
                        width: Val::Px(3.0),
                        height: Val::Px(strip_h),
                        border_radius: BorderRadius::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(edge),
                ));
                // Texts: name row (badge + name), then controller/target line.
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|name_row| {
                        name_row
                            .spawn((
                                Node {
                                    padding: UiRect::axes(Val::Px(5.0), Val::Px(1.0)),
                                    border_radius: BorderRadius::all(Val::Px(3.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(
                                    kind_color.to_srgba().red,
                                    kind_color.to_srgba().green,
                                    kind_color.to_srgba().blue,
                                    if is_trigger { 0.45 } else { 0.22 },
                                )),
                            ))
                            .with_children(|badge| {
                                badge.spawn((
                                    Text::new(kind_str),
                                    tf(if is_top { 11.0 } else { 9.0 }),
                                    TextColor(kind_color),
                                ));
                            });
                        name_row.spawn((
                            Text::new(name),
                            tf(if is_top { 14.0 } else { 12.0 }),
                            TextColor(name_color),
                        ));
                    });
                    let sub = if is_top && tgt_str.is_empty() {
                        format!("{ctrl_str}  ·  resolves next")
                    } else if is_top {
                        format!("{ctrl_str}  {tgt_str}  ·  resolves next")
                    } else if tgt_str.is_empty() {
                        ctrl_str.clone()
                    } else {
                        format!("{ctrl_str}  {tgt_str}")
                    };
                    col.spawn((
                        Text::new(sub),
                        tf(if is_top { 11.0 } else { 10.0 }),
                        TextColor(if is_top { theme::ACCENT_GOLD } else { theme::TEXT_SECONDARY }),
                    ));
                });
            });
        }

        // Footer: in-context priority affordance.
        if your_priority {
            p.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                    justify_content: JustifyContent::Center,
                    align_self: AlignSelf::Center,
                    margin: UiRect::top(Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                crate::theme::HoverTint::new(theme::BUTTON_PRIMARY_BG),
                StackResolveButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Let resolve ▶   (Space)"),
                    tf(12.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
        } else {
            p.spawn((
                Text::new(format!("Waiting for {priority_name}…")),
                tf(11.0),
                TextColor(theme::TEXT_MUTED),
            ));
        }
    });
}
