//! `CRAB_PERF=1`: a frame-time readout in the window's top-right corner, and
//! the same numbers in the log once a second.
//!
//! Frames per second here are frames *drawn*: with frame pacing
//! (`systems::frame_pacing`) an idle table reads ~4 and a busy one the
//! display's refresh rate, which is the thing to watch when a change keeps
//! the loop awake. `CRAB_PERF=changes` also logs, for every frame that has
//! any, which components changed and on what — the per-frame writer that
//! holds the loop at full rate shows up by name. Off by default, and nothing
//! is added to the app then.

use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
};
use bevy::prelude::*;

pub struct PerfHudPlugin;

impl Plugin for PerfHudPlugin {
    fn build(&self, app: &mut App) {
        if std::env::var_os("CRAB_PERF").is_none() {
            return;
        }
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, spawn_perf_hud)
        .add_systems(Update, update_perf_hud);
        if std::env::var("CRAB_PERF").is_ok_and(|v| v == "changes") {
            app.add_systems(Last, log_changes);
        }
    }
}

#[derive(Component)]
struct PerfHudText;

fn spawn_perf_hud(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont { font_size: FontSize::Px(12.0), ..default() },
        TextColor(Color::srgba(0.8, 1.0, 0.8, 0.9)),
        Node { position_type: PositionType::Absolute, right: Val::Px(6.0), top: Val::Px(2.0), ..default() },
        GlobalZIndex(i32::MAX),
        PerfHudText,
    ));
}

/// Refreshed four times a second, and written only when it reads differently.
fn update_perf_hud(
    time: Res<Time<Real>>,
    store: Res<DiagnosticsStore>,
    mut next: Local<f32>,
    mut text: Query<&mut Text, With<PerfHudText>>,
) {
    let now = time.elapsed_secs();
    if now < *next {
        return;
    }
    *next = now + 0.25;
    let value = |path| store.get(path).and_then(|d| d.smoothed()).unwrap_or(0.0);
    let line = format!(
        "{:.0} fps · {:.1} ms · {:.0} entities",
        value(&FrameTimeDiagnosticsPlugin::FPS),
        value(&FrameTimeDiagnosticsPlugin::FRAME_TIME),
        value(&EntityCountDiagnosticsPlugin::ENTITY_COUNT),
    );
    if let Ok(mut text) = text.single_mut()
        && text.0 != line
    {
        text.0 = line;
    }
}

/// Bevy system (`Last`, `CRAB_PERF=changes`): log this frame's changed
/// components — the ones `systems::frame_pacing` reads as "something moved"
/// — counted by what the changed entity is.
fn log_changes(world: &mut World) {
    let frame = world.resource::<bevy::diagnostic::FrameCount>().0;
    let mut out: Vec<String> = Vec::new();
    macro_rules! count {
        ($($t:ty),* $(,)?) => {$({
            let mut q = world.query_filtered::<Entity, Changed<$t>>();
            let changed: Vec<Entity> = q.iter(world).collect();
            if !changed.is_empty() {
                let mut by: std::collections::BTreeMap<&'static str, usize> = Default::default();
                for e in &changed {
                    *by.entry(what(world, *e)).or_default() += 1;
                }
                let by: String = by.iter().map(|(k, v)| format!(" {k}:{v}")).collect();
                out.push(format!("{}={}{by}", stringify!($t), changed.len()));
            }
        })*};
    }
    count!(
        Transform,
        Visibility,
        Node,
        UiTransform,
        Text,
        TextColor,
        BackgroundColor,
        BorderColor,
        ImageNode,
        MeshMaterial3d<StandardMaterial>,
        Mesh3d,
    );
    if !out.is_empty() {
        info!("changes f{frame}: {}", out.join(" | "));
    }
}

/// What a changed entity is, by the first client marker it carries (the
/// component names themselves need Bevy's `debug` feature).
fn what(world: &World, e: Entity) -> &'static str {
    use crate::card::*;
    use crate::systems::*;
    let entity = world.entity(e);
    macro_rules! first {
        ($($t:ty),* $(,)?) => {$(
            if entity.contains::<$t>() {
                let name = std::any::type_name::<$t>();
                return name.rsplit("::").next().unwrap_or(name);
            }
        )*};
    }
    first!(
        crate::MainCamera,
        BattlefieldCard,
        HandCard,
        OpponentHandCard,
        StackCard,
        CommandZoneCard,
        DeckPile,
        GraveyardPile,
        ExilePile,
        counter_coins::CounterCoin,
        counter_coins::CounterLabel,
        pt_label::PtLabel,
        keyword_label::KeywordLabel,
        combat_badge::CombatChip,
        token_badge::TokenPileBadge,
        table_tint::SeatNamePlate,
        table_tint::SeatTint,
        game_ui::PhaseStepLabel,
        game_ui::PlayerHudPanel,
        game_ui::LogRow,
        ui::HoverCardPreview,
        bevy::camera::visibility::NoFrustumCulling,
    );
    if entity.contains::<Text>() {
        "text"
    } else if entity.contains::<Node>() {
        "ui node"
    } else if entity.contains::<Mesh3d>() {
        "mesh"
    } else {
        "other"
    }
}
