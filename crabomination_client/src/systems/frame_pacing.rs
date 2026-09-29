//! Frame pacing: draw a frame when something on screen changes, not at the
//! monitor's refresh rate forever.
//!
//! A card game's table is still most of the time, and Bevy's default
//! (`WinitSettings::game()`) renders continuously while the window has focus
//! — 144 or 240 full frames a second of shadows, bloom and SMAA on a 144/240 Hz
//! display, competing with the local match's bot search for the CPU. Here the
//! loop is reactive (`UpdateMode::Reactive`) and each frame decides what the
//! next one needs:
//!
//! * **Something changed** — a card moved, a UI node, text or colour was
//!   rewritten, a material faded, card art arrived, the app state switched:
//!   [`RequestRedraw`], so the next frame follows at once and an animation
//!   runs at the full frame rate.
//! * **Only ambient light moves** — the active seat's breathing glow
//!   ([`crate::systems::glow`]) or the arrows' flowing bands
//!   ([`crate::systems::arrows`]), reported through [`LightMotion`]: the
//!   loop waits [`AMBIENT_WAIT`], ~30 fps, plenty for motion that slow.
//! * **Nothing** — it waits [`IDLE_WAIT`]. Timers read true `Time`, so
//!   anything scheduled (a clock, a delayed panel) lands within that wait.
//!
//! Input wakes the loop as it always does (winit window/device events), and
//! a server message wakes it the moment it arrives ([`wake_on_net_messages`]),
//! so an idle table answers a click or an opponent's play without the wait.
//!
//! It only works while no system rewrites a component every frame with the
//! value it already holds: that reads as a change and holds the loop at the
//! full frame rate. The overlay, pile, camera, hover and phase-chart writers
//! compare before they write for that reason (`theme::place_overlay`,
//! `animate::animate_hover_lift`, `camera_zoom`, …). Animations step by
//! `animate::anim_dt`, capped, so the frame that ends an idle wait doesn't
//! jump an animation a quarter of a second into its run.
//!
//! `CRAB_CONTINUOUS=1` restores continuous rendering; a layout-harness
//! screenshot run (`--screenshot`) always renders continuously, so its
//! timed captures stay what they were.

use std::time::Duration;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use bevy::window::RequestRedraw;
use bevy::winit::{UpdateMode, WinitSettings};

/// Wait between frames with nothing changing and no ambient motion, focused.
pub const IDLE_WAIT: Duration = Duration::from_millis(250);
/// Wait between frames while only ambient light moves, focused (~30 fps).
pub const AMBIENT_WAIT: Duration = Duration::from_micros(33_333);
/// The same two waits while the window is out of focus.
const UNFOCUSED_IDLE_WAIT: Duration = Duration::from_secs(1);
const UNFOCUSED_AMBIENT_WAIT: Duration = Duration::from_millis(100);

/// Set by the immediate-mode light renderers ([`crate::systems::glow`],
/// [`crate::systems::arrows`]), whose meshes change every frame they show
/// and so can't be told apart by change detection. Read and cleared by
/// [`pace_frames`].
#[derive(Resource, Default)]
pub struct LightMotion {
    /// Something drawn this frame moves on its own and slowly — the seat
    /// glow's breathing, an arrow's flowing bands: ~30 fps is enough.
    pub ambient: bool,
    /// Something drawn this frame is a short animation — an arrow growing
    /// out of its source or fading away: full frame rate.
    pub transient: bool,
}

pub struct FramePacingPlugin {
    /// Render continuously regardless (a harness screenshot run).
    pub continuous: bool,
}

impl Plugin for FramePacingPlugin {
    fn build(&self, app: &mut App) {
        let enabled = !self.continuous && std::env::var_os("CRAB_CONTINUOUS").is_none();
        app.init_resource::<LightMotion>();
        if !enabled {
            return;
        }
        app.insert_resource(settings(false))
            .add_systems(Last, pace_frames);
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            PreUpdate,
            wake_on_net_messages
                .run_if(resource_exists_and_changed::<crate::net_plugin::NetInbox>)
                .before(crate::net_plugin::poll_net),
        );
    }
}

fn settings(ambient: bool) -> WinitSettings {
    let (focused, unfocused) =
        if ambient { (AMBIENT_WAIT, UNFOCUSED_AMBIENT_WAIT) } else { (IDLE_WAIT, UNFOCUSED_IDLE_WAIT) };
    WinitSettings {
        focused_mode: UpdateMode::reactive(focused),
        unfocused_mode: UpdateMode::reactive_low_power(unfocused),
    }
}

/// Everything whose change this frame means the next frame must follow at
/// once. The immediate-mode light meshes are left out (`NoFrustumCulling`):
/// they are rebuilt every frame they show, and [`LightMotion`] paces them.
#[derive(bevy::ecs::system::SystemParam)]
#[allow(clippy::type_complexity)]
pub struct Changes<'w, 's> {
    transforms: Query<'w, 's, (), Changed<Transform>>,
    visibility: Query<'w, 's, (), Changed<Visibility>>,
    nodes: Query<'w, 's, (), Or<(Changed<Node>, Changed<ScrollPosition>)>>,
    ui_transforms: Query<'w, 's, (), Changed<UiTransform>>,
    texts: Query<'w, 's, (), Or<(Changed<Text>, Changed<TextColor>, Changed<TextFont>)>>,
    colours: Query<'w, 's, (), Or<(Changed<BackgroundColor>, Changed<BorderColor>, Changed<ImageNode>)>>,
    materials: Query<'w, 's, (), Changed<MeshMaterial3d<StandardMaterial>>>,
    meshes: Query<'w, 's, (), (Changed<Mesh3d>, Without<NoFrustumCulling>)>,
    /// Death bursts, sparks and mana motes draw through the glow mesh; their
    /// own clocks say they are running.
    impacts: Query<
        'w,
        's,
        (),
        Or<(Changed<crate::systems::impact::Impact>, Changed<crate::systems::impact::ManaMote>)>,
    >,
}

impl Changes<'_, '_> {
    fn any(&self) -> bool {
        !(self.transforms.is_empty()
            && self.visibility.is_empty()
            && self.nodes.is_empty()
            && self.ui_transforms.is_empty()
            && self.texts.is_empty()
            && self.colours.is_empty()
            && self.materials.is_empty()
            && self.meshes.is_empty()
            && self.impacts.is_empty())
    }
}

/// Bevy system (`Last`): decide when the next frame comes. See the module.
fn pace_frames(
    changes: Changes,
    mut material_events: MessageReader<AssetEvent<StandardMaterial>>,
    mut image_events: MessageReader<AssetEvent<Image>>,
    state: Res<State<crate::menu::AppState>>,
    mut lights: ResMut<LightMotion>,
    mut winit: ResMut<WinitSettings>,
    mut redraw: MessageWriter<RequestRedraw>,
) {
    // Both readers are read to the end (`fold`, not `any`, which stops at the
    // first hit): a message left unread would count again next frame.
    let materials_moved =
        material_events.read().fold(false, |moved, e| moved | !matches!(e, AssetEvent::Unused { .. }));
    let art_arrived = image_events.read().fold(false, |arrived, e| {
        arrived
            | matches!(
                e,
                AssetEvent::Added { .. } | AssetEvent::Modified { .. } | AssetEvent::LoadedWithDependencies { .. }
            )
    });
    let LightMotion { ambient, transient } = std::mem::take(&mut *lights);
    if transient || changes.any() || materials_moved || art_arrived || state.is_changed() {
        redraw.write(RequestRedraw);
    }
    let want = settings(ambient);
    if winit.focused_mode != want.focused_mode || winit.unfocused_mode != want.unfocused_mode {
        *winit = want;
    }
}

/// Bevy system: when a [`NetInbox`](crate::net_plugin::NetInbox) is
/// installed, put a relay thread in front of its receiver that wakes the
/// frame loop for every message.
///
/// The inbox is filled by some ten different threads (the local match, the
/// TCP and WebSocket readers, lobby and rematch sessions), and a reactive
/// loop would otherwise see their messages only at its next idle wake — up
/// to [`IDLE_WAIT`] late. The relay keeps the inbox's contract: messages in
/// order, and the disconnect is passed on when the far end hangs up.
/// Swapping the receiver goes through the inbox's own `Mutex`, so this
/// system reads the resource and does not re-trigger itself.
#[cfg(not(target_arch = "wasm32"))]
fn wake_on_net_messages(
    inbox: Res<crate::net_plugin::NetInbox>,
    proxy: Option<Res<bevy::winit::EventLoopProxyWrapper>>,
) {
    use std::sync::mpsc;
    let Some(proxy) = proxy else { return };
    let proxy: bevy::winit::EventLoopProxy<bevy::winit::WinitUserEvent> = (**proxy).clone();
    let (tx, rx) = mpsc::channel();
    // The upstream receiver is handed to the relay only once it is running:
    // if the thread can't start, the inbox is left exactly as it was.
    let (hand_over, handed) = mpsc::channel();
    let spawned = std::thread::Builder::new().name("net-wake".into()).spawn(move || {
        let Ok(upstream) = handed.recv() else { return };
        let upstream: mpsc::Receiver<_> = upstream;
        // Ends when the sender hangs up (the disconnect reaches the inbox
        // as `tx` drops) or when the inbox is dropped (the send fails).
        while let Ok(msg) = upstream.recv() {
            if tx.send(msg).is_err() {
                break;
            }
            let _ = proxy.send_event(bevy::winit::WinitUserEvent::WakeUp);
        }
    });
    match spawned {
        Ok(_) => {
            let mut guard = inbox.0.lock().unwrap_or_else(|p| p.into_inner());
            let upstream = std::mem::replace(&mut *guard, rx);
            let _ = hand_over.send(upstream);
        }
        Err(e) => {
            warn!("frame pacing: no relay for the net inbox ({e}); server messages wait for the next idle wake");
        }
    }
}
