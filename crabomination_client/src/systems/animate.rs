use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;

use bevy::prelude::*;

use bevy::ecs::system::SystemParam;

use crate::card::{
    hand_card_transform, layout::player_hand_anchor, Animating, BattlefieldCard,
    CardHoverLift, CardMeshAssets, CombatLurch, DeathBeat, DrawCardAnimation, FrontFaceMesh,
    GameCardId, HandCard, HandSlideAnimation, MdfcFlipAnimation, PlayCardAnimation,
    ReturnToDeckAnimation, ReturnToHandAnimation, RevealPeekAnimation, SendToGraveyardAnimation,
    TapAnimation, Vanishing, BF_HOVER_GROW, BF_HOVER_LIFT, CARD_WIDTH, DEATH_BEAT_SECS,
    HOVER_LIFT_SPEED,
};
use crate::net_plugin::CurrentView;
use crabomination::card::CardId;

/// Global animation playback speed multiplier (1.0 = normal, 2.0 = double speed, etc.).
#[derive(Resource)]
pub struct AnimationSpeed(pub f32);

impl Default for AnimationSpeed {
    fn default() -> Self { AnimationSpeed(1.0) }
}

/// The longest step a single frame may advance an animation, in seconds.
pub const MAX_ANIM_STEP: f32 = 1.0 / 30.0;

/// This frame's step for an animation: the frame time, capped at
/// [`MAX_ANIM_STEP`].
///
/// The frame loop idles between changes (`systems::frame_pacing`), so the
/// frame that wakes it — a server message, a click — can come a quarter of
/// a second after the last one, and an animation started on it would jump
/// that far into its run before its first frame is drawn. `Time` itself
/// stays true (the rope and chess clocks read it); only animation steps are
/// capped. Below 30 fps an animation plays a little slower rather than
/// skipping.
pub fn anim_dt(time: &Time) -> f32 {
    time.delta_secs().min(MAX_ANIM_STEP)
}


pub fn ease_in_out(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Smoothly lerp each card's Y toward its target (base + hover lift).
/// Skipped for any card that has another animation currently driving its transform.
#[allow(clippy::type_complexity)]
pub fn animate_hover_lift(
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<
        (&mut Transform, &mut CardHoverLift, Option<&BattlefieldCard>),
        (
            Without<DrawCardAnimation>,
            Without<HandSlideAnimation>,
            Without<PlayCardAnimation>,
            Without<TapAnimation>,
            Without<SendToGraveyardAnimation>,
            Without<ReturnToDeckAnimation>,
            Without<ReturnToHandAnimation>,
            Without<RevealPeekAnimation>,
        ),
    >,
) {
    let dt = anim_dt(&time) * speed.0;
    for (mut transform, mut lift, battlefield) in &mut cards {
        // Every write below is guarded: a settled card must not touch its
        // `Transform` or its `CardHoverLift`, or it is re-propagated and
        // re-extracted every frame and keeps the reactive frame loop awake
        // (`systems::frame_pacing`). The ease snaps once it is within a
        // hair of its target — an exponential approach never quite lands.
        if lift.current_lift != lift.target_lift {
            let spd = HOVER_LIFT_SPEED * dt;
            let next = lift.current_lift + (lift.target_lift - lift.current_lift) * spd.min(1.0);
            lift.current_lift =
                if (lift.target_lift - next).abs() < 1e-4 { lift.target_lift } else { next };
        }
        let at = lift.base_translation + Vec3::Y * lift.current_lift;
        if transform.translation != at {
            transform.translation = at;
        }
        // A battlefield card grows with its lift. (A hand card's scale is
        // the hand zoom's; a stack card's the lane's.)
        if battlefield.is_some() {
            let grown = 1.0 + BF_HOVER_GROW * (lift.current_lift / BF_HOVER_LIFT).clamp(0.0, 1.0);
            if transform.scale.x != grown {
                transform.scale = Vec3::splat(grown);
            }
        }
    }
}

/// How far a hovered battlefield card turns its face toward the camera at
/// full lift.
const BF_HOVER_TILT: f32 = 0.14;

/// The turn a hovered battlefield card's face takes toward the camera, on
/// top of the rotation the rest of the game gives it. Recorded so it can be
/// taken off again: a card's rotation belongs to the layout, the tap
/// animation and more, which write it outright, and a tilt folded into it
/// would compound frame on frame. [`untilt_hovered_cards`] takes the tilt off
/// before `Update`, so every other system sees and writes the card's own
/// rotation; [`tilt_hovered_cards`] puts it back after them, before the
/// transforms propagate — no pivot entity between a card and its face, and
/// the badges, borders and chips parented to it tilt with it.
///
/// Both halves run every frame for a lifted card, so they keep the rotation
/// exact and silent when nothing moved: `base` is the card's own rotation as
/// the rest of the game left it (restored bit for bit, not by multiplying
/// the tilt back out), and `shown` is the rotation it was last drawn with. A
/// still, hovered card then writes nothing that change detection sees — or
/// it would re-propagate and re-extract every frame and keep the reactive
/// frame loop (`systems::frame_pacing`) awake while the pointer rests on it.
#[derive(Component)]
pub struct HoverTilt {
    /// The tilt currently folded into the rotation (identity once
    /// [`untilt_hovered_cards`] has taken it off).
    tilt: Quat,
    base: Quat,
    shown: Quat,
}

/// The turn toward `eye` for a card at `at` whose face points along
/// `normal`, lifted `lift` (0-1) of the way: its face turns
/// [`BF_HOVER_TILT`] toward the eye, or straight at it if that's nearer.
pub(crate) fn hover_tilt(normal: Vec3, at: Vec3, eye: Vec3, lift: f32) -> Quat {
    let to_eye = (eye - at).normalize_or_zero();
    let axis = normal.cross(to_eye);
    if axis.length_squared() < 1e-8 || lift <= 0.0 {
        return Quat::IDENTITY;
    }
    let angle = normal.angle_between(to_eye).min(BF_HOVER_TILT) * lift.clamp(0.0, 1.0);
    Quat::from_axis_angle(axis.normalize(), angle)
}

/// Bevy system (`First`): take last frame's hover tilt off each card.
pub fn untilt_hovered_cards(mut cards: Query<(&mut Transform, &mut HoverTilt)>) {
    for (mut transform, mut tilt) in &mut cards {
        if tilt.tilt == Quat::IDENTITY {
            continue;
        }
        if transform.rotation == tilt.shown {
            // Untouched since it was tilted: put the card's own rotation back
            // exactly, unannounced. `tilt_hovered_cards` announces whatever
            // is drawn differently at the end of the frame.
            transform.bypass_change_detection().rotation = tilt.base;
        } else {
            transform.rotation = tilt.tilt.inverse() * transform.rotation;
        }
        tilt.bypass_change_detection().tilt = Quat::IDENTITY;
    }
}

/// Bevy system (`PostUpdate`, before transform propagation): tilt each
/// hovered battlefield card's face toward the camera as it lifts.
pub fn tilt_hovered_cards(
    mut commands: Commands,
    camera: Query<&GlobalTransform, With<crate::MainCamera>>,
    mut cards: Query<(Entity, &mut Transform, &CardHoverLift, Option<&mut HoverTilt>), With<BattlefieldCard>>,
) {
    let Ok(eye) = camera.single().map(GlobalTransform::translation) else { return };
    for (entity, mut transform, lift, applied) in &mut cards {
        let lifted = lift.current_lift / BF_HOVER_LIFT;
        if lifted < 0.01 {
            // Not tilted this frame. If it was drawn tilted, the silent
            // untilt above has to be published now, or the card stays drawn
            // at its old tilt.
            if let Some(mut applied) = applied
                && applied.shown != transform.rotation
            {
                transform.set_changed();
                applied.bypass_change_detection().shown = transform.rotation;
            }
            continue;
        }
        let base = transform.rotation;
        let tilt = hover_tilt(base * Vec3::Z, transform.translation, eye, lifted);
        let shown = tilt * base;
        match applied {
            Some(mut applied) => {
                if shown == applied.shown {
                    // Drawn exactly as last frame: restore it silently.
                    transform.bypass_change_detection().rotation = shown;
                } else {
                    transform.rotation = shown;
                }
                *applied.bypass_change_detection() = HoverTilt { tilt, base, shown };
            }
            None => {
                transform.rotation = shown;
                commands.entity(entity).try_insert(HoverTilt { tilt, base, shown });
            }
        }
    }
}

/// How fast a [`CombatLurch`] strike plays: `progress` advances by this per
/// second, so the full out-and-back arc resolves in ~1/this seconds (0.4 s).
const COMBAT_STRIKE_SPEED: f32 = 2.5;

/// Fraction of the clamped distance-to-defender the attacker reaches at the
/// peak of its strike arc — a firm lunge into contact without overshooting
/// onto the defender's side of the table.
const COMBAT_STRIKE_PROGRESS: f32 = 0.85;

/// Cap on how far (world units) a creature lunges toward its target. The
/// defender's anchor is most of the table away (~15 units), so without a cap
/// the strike would fling the attacker a third of the way across the board.
/// Clamping the offset to a few card widths keeps the *direction* but turns
/// the motion into a tidy lunge that returns home.
const MAX_LUNGE_DISTANCE: f32 = CARD_WIDTH * 3.0;

/// Trigger a one-shot "strike" lunge on each attacker the instant combat
/// resolves.
///
/// Combat damage is resolved inside the server's `PassPriority` handling, so
/// the client never receives a `CombatDamage` view with attackers still
/// declared — there's no step to gate a lunge on. The reliable signal is the
/// attacking→cleared transition: we remember each attacker's strike vector
/// while it's declared, and when it stops attacking *and is still on the
/// battlefield* (i.e. it survived), we insert a [`CombatLurch`] that arcs it
/// out toward the defender and back. Attackers that died/left play their own
/// exit (graveyard flight), so they're skipped here.
pub fn update_combat_lurch_targets(
    view: Res<CurrentView>,
    mut commands: Commands,
    bf_q: Query<(Entity, &GameCardId), With<BattlefieldCard>>,
    lift_q: Query<&CardHoverLift>,
    mut prev_attackers: Local<HashMap<CardId, Vec3>>,
) {
    let Some(cv) = &view.0 else {
        prev_attackers.clear();
        return;
    };
    let viewer = cv.your_seat;
    let n_seats = cv.players.len();

    let mut entity_of: HashMap<CardId, Entity> = HashMap::new();
    for (e, gid) in &bf_q {
        entity_of.insert(gid.0, e);
    }
    let on_battlefield: HashSet<CardId> = cv.battlefield.iter().map(|p| p.id).collect();

    // Each currently-declared attacker → its strike displacement vector: a
    // flat (Y=0) direction toward the defender's anchor, clamped so the lunge
    // stays a tidy jab rather than a slide across the table, scaled to the
    // strike reach. Stored so it's ready to fire the moment combat resolves
    // (when the attacking flag clears and the defender is no longer derivable).
    let mut current: HashMap<CardId, Vec3> = HashMap::new();
    for p in &cv.battlefield {
        if !p.attacking {
            continue;
        }
        let Some(&e) = entity_of.get(&p.id) else { continue };
        let base = lift_q.get(e).map(|l| l.base_translation).unwrap_or(Vec3::ZERO);
        // Defender: first seat that isn't the attacker's controller. The engine
        // doesn't expose per-attacker defenders yet, so we point at the first
        // opponent — close enough for the lunge direction.
        let def_seat = (0..n_seats).find(|&s| s != p.controller).unwrap_or(p.controller);
        let icon = player_hand_anchor(def_seat, viewer, n_seats);
        let dir =
            Vec3::new(icon.x - base.x, 0.0, icon.z - base.z).clamp_length_max(MAX_LUNGE_DISTANCE);
        current.insert(p.id, dir * COMBAT_STRIKE_PROGRESS);
    }

    // Fire a strike for every attacker that just stopped attacking but is still
    // on the battlefield (survived combat).
    for (id, offset) in prev_attackers.iter() {
        if current.contains_key(id) || !on_battlefield.contains(id) {
            continue;
        }
        if let Some(&e) = entity_of.get(id) {
            commands.entity(e).insert(CombatLurch { progress: 0.0, offset: *offset });
        }
    }

    *prev_attackers = current;
}

/// Drive each one-shot [`CombatLurch`] strike: advance `progress` 0 → 1 and add
/// a sine arc of `offset` onto the card's transform, so it lunges out toward
/// the defender at the midpoint and returns to rest, then self-removes. Runs
/// after [`animate_hover_lift`] (which rewrites the resting translation each
/// frame) and skips the same cards it does, so a card mid-tap/flight doesn't
/// get the arc piled on top of an already-moving transform.
#[allow(clippy::type_complexity)]
pub fn animate_combat_lurch(
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut commands: Commands,
    mut q: Query<
        (Entity, &mut Transform, &mut CombatLurch),
        (
            Without<DrawCardAnimation>,
            Without<HandSlideAnimation>,
            Without<PlayCardAnimation>,
            Without<TapAnimation>,
            Without<SendToGraveyardAnimation>,
            Without<ReturnToDeckAnimation>,
            Without<ReturnToHandAnimation>,
            Without<RevealPeekAnimation>,
        ),
    >,
) {
    let dt = anim_dt(&time) * speed.0;
    for (entity, mut transform, mut lurch) in &mut q {
        lurch.progress += dt * COMBAT_STRIKE_SPEED;
        if lurch.progress >= 1.0 {
            commands.entity(entity).remove::<CombatLurch>();
            continue;
        }
        let arc = (lurch.progress * PI).sin();
        transform.translation += lurch.offset * arc;
    }
}

/// Drive the MDFC 180° flip: rotate the parent around its local Y axis
/// from `start_rotation` to `start_rotation * Quat::from_rotation_y(PI)`
/// over `progress: 0.0..1.0`. Both faces are pre-painted with their
/// proper Scryfall images, so this rotation alone reveals the alternate
/// face. After two flips (each +180°) the parent has rotated 360° and
/// is back at its original orientation.
#[allow(clippy::type_complexity)]
pub fn animate_mdfc_flip(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut MdfcFlipAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut cards {
        anim.progress += anim_dt(&time) * speed.0 * anim.speed;
        let t = anim.progress.min(1.0);
        let eased = ease_in_out(t);
        let angle = eased * PI;
        transform.rotation = anim.start_rotation * Quat::from_rotation_y(angle);

        if anim.progress >= 1.0 {
            transform.rotation = anim.start_rotation * Quat::from_rotation_y(PI);
            commands.entity(entity).remove::<MdfcFlipAnimation>();
        }
    }
}

pub fn animate_draw_card(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    view: Res<CurrentView>,
    hand_zoom: Res<crate::card::HandZoom>,
    mut cards: Query<(
        Entity,
        &mut Transform,
        &mut DrawCardAnimation,
        &mut CardHoverLift,
        &HandCard,
    )>,
    all_hand_cards: Query<&HandCard>,
) {
    let total = all_hand_cards.iter().count();
    let (viewer, n_seats) = view
        .0
        .as_ref()
        .map(|cv| (cv.your_seat, cv.players.len()))
        .unwrap_or((0, 2));

    for (entity, mut transform, mut anim, mut lift, hand_card) in &mut cards {
        anim.progress += anim_dt(&time) * speed.0 * anim.speed;

        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        let arc_y = (anim.progress.clamp(0.0, 1.0) * PI).sin() * 3.0;

        let mut pos = anim.start_translation.lerp(anim.target_translation, t);
        pos.y += arc_y;
        transform.translation = pos;
        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);

        if anim.progress >= 1.0 {
            let final_t = hand_card_transform(viewer, viewer, n_seats, hand_card.slot, total, hand_zoom.0);
            transform.translation = final_t.translation;
            transform.rotation = final_t.rotation;
            transform.scale = final_t.scale;
            lift.base_translation = final_t.translation;
            lift.current_lift = 0.0;
            lift.target_lift = 0.0;
            commands.entity(entity).remove::<DrawCardAnimation>().remove::<Animating>();
        }
    }
}

pub fn animate_hand_slide(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut HandSlideAnimation, &mut CardHoverLift)>,
) {
    for (entity, mut transform, mut anim, mut lift) in &mut cards {
        anim.progress += anim_dt(&time) * speed.0 * anim.speed;
        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        let pos = anim.start_translation.lerp(anim.target_translation, t);
        transform.translation = pos;
        transform.rotation = anim.target_rotation;

        if anim.progress >= 1.0 {
            transform.translation = anim.target_translation;
            lift.base_translation = anim.target_translation;
            commands.entity(entity).remove::<HandSlideAnimation>().remove::<Animating>();
        }
    }
}

/// Animate a card moving from hand to battlefield.
pub fn animate_play_card(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(
        Entity,
        &mut Transform,
        &mut PlayCardAnimation,
        &mut CardHoverLift,
    )>,
) {
    for (entity, mut transform, mut anim, mut lift) in &mut cards {
        anim.progress += anim_dt(&time) * speed.0 * anim.speed;

        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        let arc_y = (anim.progress.clamp(0.0, 1.0) * PI).sin() * 2.0;

        let mut pos = anim.start_translation.lerp(anim.target_translation, t);
        pos.y += arc_y;
        transform.translation = pos;
        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);
        // Smooth scale from hand size → where it lands (1 on the table).
        // Hand zoom can make `start_scale` > 1 on small displays.
        let scale = anim.start_scale + (anim.target_scale - anim.start_scale) * t;
        transform.scale = Vec3::splat(scale);

        if anim.progress >= 1.0 {
            transform.translation = anim.target_translation;
            transform.rotation = anim.target_rotation;
            transform.scale = Vec3::splat(anim.target_scale);
            lift.base_translation = anim.target_translation;
            lift.current_lift = 0.0;
            lift.target_lift = 0.0;
            commands.entity(entity).remove::<PlayCardAnimation>().remove::<Animating>();
        }
    }
}

/// How far a dying card shrinks over its [`DeathBeat`].
const DEATH_SHRINK: f32 = 0.1;
/// Seconds a leaving token takes to shrink away.
const VANISH_SECS: f32 = 0.3;

/// A dying card's face colour `k` (0-1) of the way through its
/// [`DeathBeat`]: flushing ember-red, then charring to near black. Linear,
/// as the face material's own white is.
pub(crate) fn char_tint(k: f32) -> Color {
    // How far through the beat the face is reddest.
    const FLUSH: f32 = 0.35;
    let white = LinearRgba::WHITE;
    let ember = LinearRgba::rgb(1.0, 0.42, 0.3);
    let ash = LinearRgba::rgb(0.1, 0.05, 0.04);
    let k = k.clamp(0.0, 1.0);
    let tint = if k < FLUSH { white.mix(&ember, k / FLUSH) } else { ember.mix(&ash, (k - FLUSH) / (1.0 - FLUSH)) };
    Color::LinearRgba(tint)
}

/// Each card's face material, for tinting a dying card.
#[derive(SystemParam)]
pub struct CardFaces<'w, 's> {
    faces: Query<'w, 's, &'static MeshMaterial3d<StandardMaterial>, With<FrontFaceMesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    assets: Option<Res<'w, CardMeshAssets>>,
}

impl CardFaces<'_, '_> {
    /// Multiply the face among `children` by `tint`. A face-down card shows
    /// the shared card back, which it leaves alone: tinting it would tint
    /// every card back on the table.
    fn tint(&mut self, children: &Children, tint: Color) {
        let back = self.assets.as_ref().map(|a| a.back_material.id());
        for child in children.iter() {
            let Ok(face) = self.faces.get(child) else { continue };
            if Some(face.0.id()) == back {
                continue;
            }
            if let Some(mut material) = self.materials.get_mut(&face.0) {
                material.base_color = tint;
            }
        }
    }
}

/// Advance a dying card's [`DeathBeat`] by `dt`: darken its face and shrink
/// it. `true` while the beat holds the card on the table.
fn beat_death(beat: &mut DeathBeat, dt: f32, transform: &mut Transform, children: Option<&Children>, faces: &mut CardFaces) -> bool {
    if beat.age >= DEATH_BEAT_SECS {
        return false;
    }
    beat.age += dt;
    let k = ease_in_out((beat.age / DEATH_BEAT_SECS).min(1.0));
    transform.scale = Vec3::splat(1.0 - DEATH_SHRINK * k);
    if let Some(children) = children {
        faces.tint(children, char_tint(k));
    }
    true
}

/// Animate a card flying to the graveyard with a tumbling spin, then despawn
/// it. A card that died plays its [`DeathBeat`] where it lay first.
#[allow(clippy::type_complexity)]
pub fn animate_send_to_graveyard(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut SendToGraveyardAnimation, Option<&mut DeathBeat>, Option<&Children>)>,
    mut faces: CardFaces,
) {
    let dt = anim_dt(&time) * speed.0;
    for (entity, mut transform, mut anim, beat, children) in &mut cards {
        let died = beat.is_some();
        if let Some(mut beat) = beat
            && beat_death(&mut beat, dt, &mut transform, children, &mut faces)
        {
            // Held where it lay: the death may have landed a frame after
            // the flight began.
            anim.progress = 0.0;
            transform.translation = anim.start_translation;
            transform.rotation = anim.start_rotation;
            continue;
        }
        anim.progress += dt * anim.speed;

        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        // Arc: rise then fall as the card travels to the graveyard
        let arc_y = (anim.progress.clamp(0.0, 1.0) * PI).sin() * 2.0;
        let mut pos = anim.start_translation.lerp(anim.target_translation, t);
        pos.y += arc_y;
        transform.translation = pos;
        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);
        // A dying card regains its size on the way.
        if died {
            transform.scale = Vec3::splat(1.0 - DEATH_SHRINK * (1.0 - t));
        }

        if anim.progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Shrink each [`Vanishing`] token away where it lay, after its
/// [`DeathBeat`] if it died, then despawn it.
pub fn animate_vanishing(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut Vanishing, Option<&mut DeathBeat>, Option<&Children>)>,
    mut faces: CardFaces,
) {
    let dt = anim_dt(&time) * speed.0;
    for (entity, mut transform, mut vanish, beat, children) in &mut cards {
        let size = if beat.is_some() { 1.0 - DEATH_SHRINK } else { 1.0 };
        if let Some(mut beat) = beat
            && beat_death(&mut beat, dt, &mut transform, children, &mut faces)
        {
            continue;
        }
        vanish.age += dt;
        let k = (vanish.age / VANISH_SECS).min(1.0);
        transform.scale = Vec3::splat(size * (1.0 - k * k));
        if k >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// A creature that was just dealt damage shudders: a quick side-to-side
/// shake that dies away, bigger for a bigger hit.
#[derive(Component)]
pub struct Jolt {
    age: f32,
    reach: f32,
}

/// How long a [`Jolt`] lasts, how fast it shakes, and how far: `JOLT_REACH`
/// plus `JOLT_PER_DAMAGE` a point of damage, up to six.
const JOLT_SECS: f32 = 0.34;
const JOLT_HZ: f32 = 9.0;
const JOLT_REACH: f32 = 0.08;
const JOLT_PER_DAMAGE: f32 = 0.025;

impl Jolt {
    pub fn new(damage: u32) -> Self {
        Self { age: 0.0, reach: JOLT_REACH + JOLT_PER_DAMAGE * damage.min(6) as f32 }
    }
}

/// How far across (world units) a card `age` seconds into a jolt of `reach`
/// is thrown: kicked at once, swinging back and forth, dying away.
fn jolt_offset(age: f32, reach: f32) -> f32 {
    let k = age / JOLT_SECS;
    if !(0.0..1.0).contains(&k) {
        return 0.0;
    }
    reach * (1.0 - k) * (1.0 - k) * (std::f32::consts::TAU * JOLT_HZ * age).sin()
}

/// Bevy system: shake each jolted battlefield card, on top of the place
/// `animate_hover_lift` put it this frame (so the shake never accumulates).
#[allow(clippy::type_complexity)]
pub fn animate_jolt(
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut commands: Commands,
    mut cards: Query<
        (Entity, &mut Transform, &mut Jolt),
        (
            With<BattlefieldCard>,
            With<CardHoverLift>,
            Without<DrawCardAnimation>,
            Without<HandSlideAnimation>,
            Without<PlayCardAnimation>,
            Without<TapAnimation>,
            Without<SendToGraveyardAnimation>,
            Without<ReturnToDeckAnimation>,
            Without<ReturnToHandAnimation>,
            Without<RevealPeekAnimation>,
        ),
    >,
) {
    let dt = anim_dt(&time) * speed.0;
    for (entity, mut transform, mut jolt) in &mut cards {
        jolt.age += dt;
        if jolt.age >= JOLT_SECS {
            commands.entity(entity).remove::<Jolt>();
            continue;
        }
        transform.translation.x += jolt_offset(jolt.age, jolt.reach);
    }
}

/// Animate a hand card flying back to the deck during a mulligan, then
/// despawn it. The face-down deck pile (`DeckPile`) is sized off
/// `library.size` and is updated by `sync_game_visuals`, so the visual deck
/// height already reflects the returned card; keeping a separate per-card
/// entity around would just clutter the deck pile with duplicates.
pub fn animate_return_to_deck(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut ReturnToDeckAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut cards {
        anim.progress += anim_dt(&time) * speed.0 * anim.speed;

        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        let arc_y = (anim.progress.clamp(0.0, 1.0) * PI).sin() * 2.0;
        let mut pos = anim.start_translation.lerp(anim.target_translation, t);
        pos.y += arc_y;
        transform.translation = pos;
        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);

        if anim.progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Animate a permanent flying back to its owner's hand. On completion
/// the entity either becomes a `HandCard` (viewer's bounce target —
/// keep it on screen face-up) or despawns (opponent's bounce target —
/// the next sync frame re-spawns it as a face-down `OpponentHandCard`).
pub fn animate_return_to_hand(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut ReturnToHandAnimation, Option<&mut CardHoverLift>)>,
) {
    for (entity, mut transform, mut anim, mut lift) in &mut cards {
        anim.progress += anim_dt(&time) * speed.0 * anim.speed;
        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        let arc_y = (anim.progress.clamp(0.0, 1.0) * PI).sin() * 2.5;
        let mut pos = anim.start_translation.lerp(anim.target_translation, t);
        pos.y += arc_y;
        transform.translation = pos;
        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);

        if anim.progress >= 1.0 {
            transform.translation = anim.target_translation;
            transform.rotation = anim.target_rotation;
            if anim.to_viewer {
                // Card was on the battlefield (scale 1.0) and is
                // becoming a viewer hand card; pick up whatever scale
                // the rest of the hand currently uses. The target was
                // computed with the current hand_zoom factor, so
                // snapping to its scale keeps the bounced card visually
                // consistent with neighbours.
                transform.scale = Vec3::splat(anim.target_scale);
                if let Some(ref mut l) = lift {
                    l.base_translation = anim.target_translation;
                    l.current_lift = 0.0;
                    l.target_lift = 0.0;
                }
                commands
                    .entity(entity)
                    .insert(HandCard { slot: anim.target_slot })
                    .remove::<ReturnToHandAnimation>()
                    .remove::<Animating>();
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Animate a card tapping (90°) or untapping.
pub fn animate_tap(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut TapAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut cards {
        anim.progress += anim_dt(&time) * speed.0 * anim.speed;
        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);

        if anim.progress >= 1.0 {
            transform.rotation = anim.target_rotation;
            commands.entity(entity).remove::<TapAnimation>().remove::<Animating>();
        }
    }
}

/// Three-phase animation: flip to face-up (0-0.35), hold (0.35-0.65), flip back (0.65-1.0).
pub fn animate_reveal_peek(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut RevealPeekAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut cards {
        anim.progress += anim_dt(&time) * speed.0 * anim.speed;
        let p = anim.progress.clamp(0.0, 1.0);

        let (rot_t, y_t) = if p < 0.25 {
            // Phase 1: flip to face-up
            let t = ease_in_out(p / 0.25);
            (t, t)
        } else if p < 0.75 {
            // Phase 2: hold face-up
            (1.0_f32, 1.0_f32)
        } else {
            // Phase 3: flip back
            let t = ease_in_out((p - 0.75) / 0.25);
            (1.0 - t, 1.0 - t)
        };

        transform.rotation = anim.start_rotation.slerp(anim.face_up_rotation, rot_t);
        let lift = (y_t * PI).sin().abs() * (CARD_WIDTH / 2.0);
        transform.translation.y = anim.start_y + lift;

        if anim.progress >= 1.0 {
            transform.rotation = anim.start_rotation;
            transform.translation.y = anim.start_y;
            commands.entity(entity).remove::<RevealPeekAnimation>().remove::<Animating>();
        }
    }
}

/// Keyboard shortcuts to adjust animation playback speed.
/// [ = half speed, ] = double speed, Backslash = reset to 1×.
pub fn adjust_animation_speed(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut speed: ResMut<AnimationSpeed>,
) {
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        speed.0 = (speed.0 * 0.5).max(ANIM_SPEED_MIN);
    }
    if keyboard.just_pressed(KeyCode::BracketRight) {
        speed.0 = (speed.0 * 2.0).min(ANIM_SPEED_MAX);
    }
    if keyboard.just_pressed(KeyCode::Backslash) {
        speed.0 = 1.0;
    }
}

/// User-facing range for the animation-speed slider in the HUD. The
/// keyboard shortcuts ([, ], \\) clamp to the same bounds.
pub const ANIM_SPEED_MIN: f32 = 0.1;
pub const ANIM_SPEED_MAX: f32 = 10.0;

// ── Animation queueing ───────────────────────────────────────────────────────

#[cfg(test)]
mod death_and_jolt_tests {
    use super::*;

    #[test]
    fn a_dying_card_flushes_then_chars() {
        let white = char_tint(0.0).to_linear();
        let flushed = char_tint(0.35).to_linear();
        let charred = char_tint(1.0).to_linear();
        assert_eq!(white, LinearRgba::WHITE);
        // Red-hot: the red held, the rest dropping...
        assert!(flushed.red > 0.9 && flushed.green < 0.5 && flushed.blue < 0.5);
        // ...then all but black.
        assert!(charred.red < 0.15 && charred.green < 0.1);
    }

    #[test]
    fn a_jolt_kicks_at_once_and_dies_away() {
        let reach = Jolt::new(3).reach;
        assert!((reach - (JOLT_REACH + 3.0 * JOLT_PER_DAMAGE)).abs() < 1e-6);
        // A bigger hit throws it further, to a point.
        assert!(Jolt::new(6).reach > reach && Jolt::new(20).reach == Jolt::new(6).reach);
        // Out a good way within the first swing...
        let first_swing = 1.0 / (4.0 * JOLT_HZ);
        assert!(jolt_offset(first_swing, reach) > reach * 0.6);
        // ...swinging back past the middle...
        assert!(jolt_offset(3.0 * first_swing, reach) < 0.0);
        // ...and still by the end.
        assert_eq!(jolt_offset(JOLT_SECS, reach), 0.0);
        assert!(jolt_offset(JOLT_SECS * 0.9, reach).abs() < reach * 0.02);
    }
}

#[cfg(test)]
mod hover_tilt_tests {
    use super::*;

    #[test]
    fn a_hovered_card_turns_its_face_toward_the_camera() {
        let (normal, at, eye) = (Vec3::Y, Vec3::ZERO, Vec3::new(0.0, 30.0, 20.0));
        let to_eye = eye.normalize();
        let tilted = hover_tilt(normal, at, eye, 1.0) * normal;
        // A full lift turns the face BF_HOVER_TILT toward the eye...
        assert!((normal.angle_between(tilted) - BF_HOVER_TILT).abs() < 1e-4);
        assert!(tilted.angle_between(to_eye) < normal.angle_between(to_eye));
        // ...half a lift, half as far; none, not at all.
        let half = hover_tilt(normal, at, eye, 0.5) * normal;
        assert!((normal.angle_between(half) - BF_HOVER_TILT / 2.0).abs() < 1e-4);
        assert_eq!(hover_tilt(normal, at, eye, 0.0), Quat::IDENTITY);
        // A card already facing the eye stays put.
        assert_eq!(hover_tilt(to_eye, at, eye, 1.0), Quat::IDENTITY);
    }
}
