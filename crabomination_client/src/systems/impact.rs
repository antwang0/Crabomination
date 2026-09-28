//! Transient "impact" feedback for high-drama moments: a flash, a shockwave
//! and rising embers when a permanent dies (while the card itself burns and
//! shrinks, `animate::animate_send_to_graveyard`), sparks and a shudder
//! where damage lands, a ripple for a library dig, mana flying to its spell,
//! and a red edge-vignette when the viewer loses life.
//!
//! The light is additive geometry ([`Glow`]), over-bright (HDR, past 1) so
//! it blooms on the Medium+ tiers: it reads as a flash of light rather than
//! the wireframe rings and spokes it was drawn as before. Every effect
//! self-despawns on a short timer and is tagged `InGameRoot`, so leaving the
//! match cleans them up with the rest of the HUD.

use std::collections::{HashMap, HashSet};
use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::mana::Color as ManaColor;
use crabomination::net::{GameEventWire, StackItemView};

use crate::card::layout::player_hand_anchor;
use crate::card::{BattlefieldCard, CARD_HEIGHT, CARD_WIDTH, DeathBeat, GameCardId, StackCard};
use crate::net_plugin::{CurrentView, LatestServerEvents};
use crate::systems::animate::Jolt;
use crate::systems::game_ui::InGameRoot;
use crate::systems::glow::{Glow, Light};
use crate::theme::{self, UiFonts};
use crate::MainCamera;

/// What an [`Impact`] marks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Burst {
    /// A permanent died: a flash on the table, a shockwave out across it,
    /// embers rising off the card.
    Death,
    /// Damage landed: a flash and sparks flying off.
    Spark,
    /// A library dig (explore, CR 701.40; discover, CR 701.57): a cool
    /// ripple and a few motes, so a top-of-library reveal gets noticed even
    /// when it only moves cards off the board.
    Dig,
}

impl Burst {
    fn ttl(self) -> f32 {
        match self {
            Burst::Death => DEATH_TTL,
            Burst::Spark => SPARK_TTL,
            Burst::Dig => DIG_TTL,
        }
    }
}

/// A burst of light at `at`, `age` seconds old; `seed` scatters its sparks
/// and embers.
#[derive(Component)]
pub struct Impact {
    at: Vec3,
    age: f32,
    kind: Burst,
    seed: u32,
}

/// A red screen-edge flash shown when the viewer takes life loss. Driven via
/// the node's border (centre stays clear), fading on a short timer.
#[derive(Component)]
pub struct HitVignette {
    age: f32,
    ttl: f32,
    peak_alpha: f32,
}

/// A floating "−N" numeral that punches in over a creature that was just
/// dealt damage, then rises and fades. Screen-space after spawn (no
/// per-frame card tracking). A life change's "−5" / "+3" beside a seat's HUD
/// row is one too (`game_ui::life_ticker`).
#[derive(Component)]
pub struct DamageNumeral {
    remaining: f32,
    total: f32,
    /// Spawn-time `top` in px; the numeral rises from here as it fades.
    base_top: f32,
    /// How far (px) it rises.
    rise: f32,
}

impl DamageNumeral {
    /// Rise `rise` px through `level` (UI px) instead, for a numeral that
    /// follows what it hangs from.
    pub(crate) fn rebase(&mut self, level: f32, rise: f32) {
        self.base_top = level + rise / 2.0;
        self.rise = rise;
    }
}

/// A glowing mana-coloured orb that arcs from the land that made it to the
/// spell it pays for, trailing light, and bursts on arrival — one per
/// `ManaAdded`, so a multi-pip cast reads as mana converging into the spell.
#[derive(Component)]
pub struct ManaMote {
    start: Vec3,
    end: Vec3,
    age: f32,
    colour: LinearRgba,
}

// ── Tuning ─────────────────────────────────────────────────────────────────

const DEATH_TTL: f32 = 1.0;
/// The death flash: how long it lasts and how far it spreads.
const DEATH_FLASH_SECS: f32 = 0.2;
const DEATH_FLASH_R: f32 = 2.2;
/// The death shockwave: how long it runs, and its radius at the start and
/// the end.
const DEATH_WAVE_SECS: f32 = 0.7;
const DEATH_WAVE_R: (f32, f32) = (0.5, 3.4);
const DEATH_EMBERS: u32 = 14;

const SPARK_TTL: f32 = 0.55;
const SPARK_FLASH_SECS: f32 = 0.12;
const SPARKS: u32 = 11;
/// How far a spark's streak trails behind it, in seconds of its flight.
const SPARK_STREAK: f32 = 0.06;
/// Pull on a flying spark or ember (world units / s²; embers float).
const SPARK_GRAVITY: f32 = 16.0;

const DIG_TTL: f32 = 0.6;
const DIG_WAVE_R: (f32, f32) = (0.3, 2.0);
const DIG_MOTES: u32 = 6;

/// How high a burst sits off the card or spot it marks: a shockwave or
/// ripple runs just over the flat cards around it.
const BURST_Y: f32 = 0.06;

const VIGNETTE_TTL: f32 = 0.7;
/// Border thickness (px) of the edge flash — fat enough to register in
/// peripheral vision without crowding the corner HUD panels.
const VIGNETTE_BORDER: f32 = 48.0;

const DMG_NUMERAL_SECS: f32 = 1.1;
/// How far (px) the damage numeral floats upward over its lifetime.
const DMG_NUMERAL_RISE: f32 = 40.0;
/// The numeral lands at this scale and snaps down to size over the punch.
const DMG_NUMERAL_PUNCH: f32 = 1.8;
const DMG_NUMERAL_PUNCH_SECS: f32 = 0.14;
const DMG_NUMERAL_FONT: f32 = 32.0;
/// A bright red that holds up over any card art inside its dark outline.
pub(crate) const DMG_NUMERAL_RED: Color = Color::srgb(1.0, 0.32, 0.26);
/// The outline: the numeral drawn in near-black at each of these offsets
/// (px) under the red one. The UI font is a thin Light cut, and a bare red
/// numeral all but vanished into the card art.
const DMG_NUMERAL_OUTLINE: [(f32, f32); 8] =
    [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0), (-1.4, -1.4), (1.4, -1.4), (-1.4, 1.4), (1.4, 1.4)];

/// Seconds a mana mote flies, and bursts for on arrival.
const MANA_MOTE_FLIGHT: f32 = 0.55;
const MANA_MOTE_POP: f32 = 0.16;
/// Peak arc height of a travelling mana mote.
const MANA_MOTE_LIFT: f32 = 1.6;
/// How far back a mote's trail reaches, in seconds of its flight.
const MANA_MOTE_TRAIL: f32 = 0.16;

/// Deep red-orange (HDR) for a death — blooms, then fades to nothing.
const DEATH_COLOUR: LinearRgba = LinearRgba::rgb(3.2, 0.6, 0.18);
/// An ember, hot and then cooling.
const EMBER_HOT: LinearRgba = LinearRgba::rgb(3.4, 1.4, 0.35);
const EMBER_COOL: LinearRgba = LinearRgba::rgb(0.9, 0.08, 0.02);
/// Hot, near-white spark for damage landing.
const SPARK_COLOUR: LinearRgba = LinearRgba::rgb(3.0, 2.2, 1.2);
/// Cool cyan for a library dig (explore / discover).
const DIG_COLOUR: LinearRgba = LinearRgba::rgb(0.4, 1.8, 2.6);

/// HDR mana-pip colour for a travelling mote; `None` is colorless.
fn mana_colour(color: Option<ManaColor>) -> LinearRgba {
    let (r, g, b) = match color {
        Some(ManaColor::White) => (2.7, 2.6, 2.1),
        Some(ManaColor::Blue) => (0.3, 1.4, 3.0),
        Some(ManaColor::Black) => (1.5, 0.9, 1.9),
        Some(ManaColor::Red) => (3.0, 0.7, 0.4),
        Some(ManaColor::Green) => (0.5, 2.6, 0.8),
        None => (1.8, 1.8, 2.1),
    };
    LinearRgba::rgb(r, g, b)
}

// ── Spawning ───────────────────────────────────────────────────────────────

/// Read the latest server-event batch and spawn the matching impact effects.
/// `events.0` is repopulated every frame by `poll_net` (PreUpdate) and is
/// non-empty only on a batch frame, so a plain emptiness check both gates the
/// work and prevents re-spawning across frames.
pub fn spawn_impact_effects(
    mut commands: Commands,
    events: Res<LatestServerEvents>,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    cards: Query<(Entity, &GlobalTransform, &GameCardId, Has<BattlefieldCard>)>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    ui_scale: Res<UiScale>,
    existing_vignettes: Query<Entity, With<HitVignette>>,
    mut seed: Local<u32>,
) {
    if events.0.is_empty() {
        return;
    }
    let Some(cv) = &view.0 else { return };

    let card = |id: CardId| cards.iter().find(|(_, _, g, _)| g.0 == id);
    let pos_of = |id: CardId| card(id).map(|(_, t, _, _)| t.translation());
    let mut burst = |commands: &mut Commands, at: Vec3, kind: Burst| {
        *seed = seed.wrapping_add(1);
        commands.spawn((Impact { at: at + Vec3::Y * BURST_Y, age: 0.0, kind, seed: *seed }, InGameRoot));
    };
    // Damage each struck permanent took in this batch, in the order first
    // struck: two blockers' damage is one "−N", not two numerals printed
    // over each other.
    let mut struck: Vec<(CardId, u32)> = Vec::new();
    let mut died: HashSet<CardId> = HashSet::new();

    for ev in &events.0 {
        match ev {
            // Every permanent that dies burns — once, though a creature's
            // death may come as both events.
            GameEventWire::CreatureDied { card_id } | GameEventWire::PermanentDied { card_id, .. } => {
                if !died.insert(*card_id) {
                    continue;
                }
                if let Some((entity, t, _, _)) = card(*card_id) {
                    burst(&mut commands, t.translation(), Burst::Death);
                    // The card burns before it leaves: its flight to the
                    // graveyard (or a token's vanishing) waits on this.
                    commands.entity(entity).try_insert(DeathBeat::default());
                }
            }
            GameEventWire::DamageDealt { to_card, to_player, amount } => {
                let card_world = to_card.and_then(&pos_of);
                // Spark at the struck permanent, else at the damaged player's
                // table anchor. (Unblocked combat damage and direct burn both
                // surface as a player hit here.)
                let at = card_world.or_else(|| {
                    to_player
                        .map(|seat| player_hand_anchor(seat, cv.your_seat, cv.players.len()))
                });
                if let Some(p) = at {
                    burst(&mut commands, p, Burst::Spark);
                }
                // Floating "−N" on a struck creature, so the player reads how
                // much it took. A player's life change has its own numeral
                // beside their HUD row (`game_ui::life_ticker`), so only
                // creatures here.
                if *amount > 0
                    && let Some(id) = to_card
                {
                    match struck.iter_mut().find(|(c, _)| c == id) {
                        Some((_, total)) => *total += amount,
                        None => struck.push((*id, *amount)),
                    }
                }
            }
            GameEventWire::Explored { card_id, .. } => {
                if let Some(p) = pos_of(*card_id) {
                    burst(&mut commands, p, Burst::Dig);
                }
            }
            GameEventWire::Discovered { player, .. } => {
                let at = player_hand_anchor(*player, cv.your_seat, cv.players.len());
                burst(&mut commands, at, Burst::Dig);
            }
            GameEventWire::LifeLost { player, amount } if *player == cv.your_seat => {
                // Replace any in-flight vignette so rapid hits don't stack into
                // an opaque red frame; scale the flash with the hit size.
                for e in &existing_vignettes {
                    commands.entity(e).despawn();
                }
                let peak_alpha = (0.18 + 0.05 * *amount as f32).min(0.6);
                spawn_vignette(&mut commands, peak_alpha);
            }
            _ => {}
        }
    }

    // A struck creature still on the table shudders with the hit.
    for &(id, amount) in &struck {
        if let Some((entity, _, _, true)) = card(id) {
            commands.entity(entity).try_insert(Jolt::new(amount));
        }
    }

    // Each numeral punches in on the struck card's P/T box and rises off it,
    // uncovering the toughness it just turned red. The box is also the part
    // of a back-row card that the card in front of it leaves showing: at the
    // card's centre, a back-row creature's numeral landed on its neighbour.
    let Ok((camera, cam_xform)) = camera_q.single() else { return };
    for (id, amount) in struck {
        if let Some((_, card, _, _)) = card(id)
            && let Some(screen) = crate::theme::project_to_ui(
                camera,
                cam_xform,
                &ui_scale,
                card.transform_point(crate::systems::pt_label::PT_BOX),
            )
        {
            spawn_damage_numeral(&mut commands, &ui_fonts, amount, screen);
        }
    }
}

/// Spawn a mana mote for each `ManaAdded` / `ColorlessManaAdded` in the latest
/// batch, arcing from one of the producing player's actually-tapped lands to
/// the spell (or ability) it's paying for on the stack.
///
/// Runs after `sync_game_visuals` so the freshly-cast spell's `StackCard`
/// entity already exists to aim at. When the stack is empty (mana floated with
/// nothing to feed) no motes fire — they'd otherwise sail at an empty table.
pub fn spawn_mana_motes(
    mut commands: Commands,
    events: Res<LatestServerEvents>,
    view: Res<CurrentView>,
    bf_cards: Query<(&Transform, &GameCardId), With<BattlefieldCard>>,
    stack_cards: Query<(&Transform, &GameCardId), With<StackCard>>,
    // Lands that were tapped as of the previous frame, so we can tell which
    // ones *newly* tapped in the current batch — `ManaAdded` carries no source
    // permanent, but the lands that just tapped this batch are what produced
    // the mana. Kept across frames; the view only changes on a batch.
    mut prev_tapped: Local<HashSet<CardId>>,
) {
    let Some(cv) = &view.0 else {
        prev_tapped.clear();
        return;
    };

    let land_pos: HashMap<CardId, Vec3> =
        bf_cards.iter().map(|(t, g)| (g.0, t.translation)).collect();
    let current_tapped: HashSet<CardId> = cv
        .battlefield
        .iter()
        .filter(|p| p.is_land() && p.tapped)
        .map(|p| p.id)
        .collect();

    if !events.0.is_empty() {
        let stack_pos = |id: CardId| {
            stack_cards.iter().find(|(_, g)| g.0 == id).map(|(t, _)| t.translation)
        };
        // Destination: the spell cast this batch → the top stack item → table
        // centre (only if something is on the stack). Empty stack ⇒ skip.
        let cast_id = events.0.iter().find_map(|e| match e {
            GameEventWire::SpellCast { card_id, .. } => Some(*card_id),
            _ => None,
        });
        let dest = cast_id
            .and_then(stack_pos)
            .or_else(|| match cv.stack.last() {
                Some(StackItemView::Known(k)) => stack_pos(k.source),
                _ => None,
            })
            .or_else(|| (!cv.stack.is_empty()).then_some(Vec3::new(0.0, 0.9, 0.0)));

        if let Some(mut dest) = dest {
            dest.y += 0.2;

            // Source pools per controller: the lands that *newly* tapped this
            // batch (the ones that produced this mana), with all of a player's
            // tapped lands as a fallback if we somehow saw no fresh tap (e.g.
            // mana from a non-land source).
            let mut newly: HashMap<usize, Vec<Vec3>> = HashMap::new();
            let mut all_tapped: HashMap<usize, Vec<Vec3>> = HashMap::new();
            for p in &cv.battlefield {
                if !p.is_land() || !p.tapped {
                    continue;
                }
                let Some(&pos) = land_pos.get(&p.id) else { continue };
                all_tapped.entry(p.controller).or_default().push(pos);
                if !prev_tapped.contains(&p.id) {
                    newly.entry(p.controller).or_default().push(pos);
                }
            }

            let mut next_src: HashMap<usize, usize> = HashMap::new();
            for ev in &events.0 {
                let (player, color, source) = match ev {
                    GameEventWire::ManaAdded { player, color, source } => {
                        (*player, Some(*color), *source)
                    }
                    GameEventWire::ColorlessManaAdded { player, source } => {
                        (*player, None, *source)
                    }
                    _ => continue,
                };
                // Prefer the exact producing permanent the engine now reports;
                // fall back to a newly-tapped (else any) land of the player for
                // sourceless mana (rituals, devotion / X-cost effects).
                let mut start = match source.and_then(|sid| land_pos.get(&sid).copied()) {
                    Some(pos) => pos,
                    None => {
                        let sources = newly
                            .get(&player)
                            .filter(|v| !v.is_empty())
                            .or_else(|| all_tapped.get(&player).filter(|v| !v.is_empty()));
                        let Some(sources) = sources else { continue };
                        let slot = next_src.entry(player).or_default();
                        let pos = sources[*slot % sources.len()];
                        *slot += 1;
                        pos
                    }
                };
                start.y += 0.3;
                commands.spawn((ManaMote { start, end: dest, age: 0.0, colour: mana_colour(color) }, InGameRoot));
            }
        }
    }

    *prev_tapped = current_tapped;
}

fn spawn_damage_numeral(commands: &mut Commands, fonts: &UiFonts, amount: u32, screen: Vec2) {
    spawn_numeral(commands, fonts, format!("-{amount}"), DMG_NUMERAL_RED, screen);
}

/// A numeral in the damage numerals' style — outlined, landing big, rising
/// and fading ([`animate_damage_numerals`]) — centred on `screen` (UI px).
/// Life changes use it too (`game_ui::life_ticker`).
pub(crate) fn spawn_numeral(commands: &mut Commands, fonts: &UiFonts, text: String, colour: Color, screen: Vec2) -> Entity {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(screen.y),
                left: Val::Px(screen.x),
                ..default()
            },
            // Centred on the hit, and landing big (`animate_damage_numerals`).
            UiTransform {
                translation: Val2::percent(-50.0, -50.0),
                scale: Vec2::splat(DMG_NUMERAL_PUNCH),
                ..UiTransform::IDENTITY
            },
            GlobalZIndex(theme::layer::HUD),
            Pickable::IGNORE,
            InGameRoot,
            DamageNumeral {
                remaining: DMG_NUMERAL_SECS,
                total: DMG_NUMERAL_SECS,
                base_top: screen.y,
                rise: DMG_NUMERAL_RISE,
            },
        ))
        .with_children(|p| {
            for (dx, dy) in DMG_NUMERAL_OUTLINE {
                p.spawn((
                    Text::new(text.clone()),
                    fonts.tf(DMG_NUMERAL_FONT),
                    TextColor(Color::srgb(0.06, 0.02, 0.02)),
                    Node { position_type: PositionType::Absolute, left: Val::Px(dx), top: Val::Px(dy), ..default() },
                    Pickable::IGNORE,
                ));
            }
            // Last, so it draws over its outline; in the flow, so the node
            // takes its size.
            p.spawn((Text::new(text), fonts.tf(DMG_NUMERAL_FONT), TextColor(colour), Pickable::IGNORE));
        })
        .id()
}

/// A damage numeral's scale `elapsed` seconds in: it lands at
/// [`DMG_NUMERAL_PUNCH`] and snaps to size, easing out.
fn damage_numeral_scale(elapsed: f32) -> f32 {
    let k = (elapsed / DMG_NUMERAL_PUNCH_SECS).clamp(0.0, 1.0);
    let ease = 1.0 - (1.0 - k) * (1.0 - k);
    DMG_NUMERAL_PUNCH + (1.0 - DMG_NUMERAL_PUNCH) * ease
}

fn spawn_vignette(commands: &mut Commands, peak_alpha: f32) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            bottom: Val::Px(0.0),
            border: UiRect::all(Val::Px(VIGNETTE_BORDER)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.9, 0.08, 0.08, 0.0)),
        // Above the board / HUD so the edge flash is never occluded; ignore
        // picking so it can't eat clicks.
        GlobalZIndex(theme::layer::SCREEN_FLASH),
        Pickable::IGNORE,
        InGameRoot,
        HitVignette { age: 0.0, ttl: VIGNETTE_TTL, peak_alpha },
    ));
}

// ── Animation ──────────────────────────────────────────────────────────────

/// A number (0-1) for scattering the `i`th spark or ember of burst `seed`;
/// `salt` picks which of its numbers.
fn scatter(seed: u32, i: u32, salt: u32) -> f32 {
    let mut h = seed.wrapping_mul(0x9e37_79b9) ^ i.wrapping_mul(0x85eb_ca6b) ^ salt.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    (h & 0xffff) as f32 / 65535.0
}

fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

fn scaled(colour: LinearRgba, k: f32) -> LinearRgba {
    LinearRgba::rgb(colour.red * k, colour.green * k, colour.blue * k)
}

/// The light of a `kind` burst at `at`, `age` seconds in: nothing once it
/// has run its course.
pub(crate) fn burst_lights(kind: Burst, at: Vec3, age: f32, seed: u32) -> Vec<Light> {
    let mut out = Vec::new();
    let r = |i: u32, salt: u32| scatter(seed, i, salt);
    match kind {
        Burst::Death => {
            if age < DEATH_FLASH_SECS {
                let k = 1.0 - age / DEATH_FLASH_SECS;
                out.push(Light::Pool {
                    at,
                    radius: DEATH_FLASH_R * (1.0 - 0.4 * k),
                    colour: scaled(DEATH_COLOUR, 1.4 * k * k),
                });
            }
            if age < DEATH_WAVE_SECS {
                let t = age / DEATH_WAVE_SECS;
                out.push(Light::Wave {
                    at,
                    radius: DEATH_WAVE_R.0 + (DEATH_WAVE_R.1 - DEATH_WAVE_R.0) * ease_out(t),
                    width: 0.35 + 0.45 * t,
                    colour: scaled(DEATH_COLOUR, 1.5 * (1.0 - t).powf(1.5)),
                });
            }
            // Embers rise off the card's face, drifting, cooling from
            // orange to a dull red, and go out.
            for i in 0..DEATH_EMBERS {
                let life = 0.55 + 0.4 * r(i, 0);
                if age >= life {
                    continue;
                }
                let k = age / life;
                let start = at + Vec3::new((r(i, 1) - 0.5) * CARD_WIDTH * 0.85, 0.0, (r(i, 2) - 0.5) * CARD_HEIGHT * 0.85);
                let angle = TAU * r(i, 3);
                let drift = Vec3::new(angle.cos(), 0.0, angle.sin()) * (0.3 + 0.6 * r(i, 4));
                let rise = 1.4 + 1.8 * r(i, 5);
                let colour = EMBER_HOT.mix(&EMBER_COOL, k);
                out.push(Light::Orb {
                    at: start + drift * age + Vec3::Y * (rise * age - 0.9 * age * age),
                    radius: (0.09 + 0.07 * r(i, 6)) * (1.0 - 0.5 * k),
                    colour: scaled(colour, 1.0 - k),
                });
            }
        }
        Burst::Spark => {
            if age < SPARK_FLASH_SECS {
                let k = 1.0 - age / SPARK_FLASH_SECS;
                out.push(Light::Orb { at: at + Vec3::Y * 0.3, radius: 1.1 * (0.5 + 0.5 * k), colour: scaled(SPARK_COLOUR, 1.6 * k) });
            }
            let t = age / (SPARK_TTL * 0.66);
            if t < 1.0 {
                out.push(Light::Wave {
                    at,
                    radius: 0.2 + 1.0 * ease_out(t),
                    width: 0.35,
                    colour: scaled(SPARK_COLOUR, 0.8 * (1.0 - t)),
                });
            }
            // Sparks fly off, up and out, streaking, and fall.
            for i in 0..SPARKS {
                let life = 0.3 + 0.25 * r(i, 0);
                if age >= life {
                    continue;
                }
                let k = age / life;
                let angle = TAU * (i as f32 + 0.8 * r(i, 1)) / SPARKS as f32;
                let dir = Vec3::new(angle.cos(), 0.35 + 0.9 * r(i, 2), angle.sin()).normalize();
                let speed = 5.0 + 5.0 * r(i, 3);
                let flown = |t: f32| at + dir * speed * t - Vec3::Y * 0.5 * SPARK_GRAVITY * t * t;
                out.push(Light::Trail {
                    points: vec![flown(age), flown((age - SPARK_STREAK).max(0.0))],
                    half: 0.08,
                    colour: scaled(SPARK_COLOUR, 2.4 * (1.0 - k)),
                });
            }
        }
        Burst::Dig => {
            let t = age / DIG_TTL;
            if t < 1.0 {
                out.push(Light::Wave {
                    at,
                    radius: DIG_WAVE_R.0 + (DIG_WAVE_R.1 - DIG_WAVE_R.0) * ease_out(t),
                    width: 0.6,
                    colour: scaled(DIG_COLOUR, 1.0 - t),
                });
            }
            for i in 0..DIG_MOTES {
                let life = 0.45 + 0.15 * r(i, 0);
                if age >= life {
                    continue;
                }
                let k = age / life;
                let angle = TAU * (i as f32 + r(i, 1)) / DIG_MOTES as f32;
                let start = at + Vec3::new(angle.cos(), 0.0, angle.sin()) * (0.4 + 0.5 * r(i, 2));
                out.push(Light::Orb {
                    at: start + Vec3::Y * (1.2 + 0.8 * r(i, 3)) * age,
                    radius: 0.08,
                    colour: scaled(DIG_COLOUR, 1.2 * (1.0 - k)),
                });
            }
        }
    }
    out
}

/// Bevy system: age each [`Impact`] and push its light, despawning it once
/// it has run its course.
pub fn animate_impacts(
    mut commands: Commands,
    time: Res<Time>,
    mut impacts: Query<(Entity, &mut Impact)>,
    mut glow: ResMut<Glow>,
) {
    for (entity, mut impact) in &mut impacts {
        impact.age += time.delta_secs();
        if impact.age >= impact.kind.ttl() {
            commands.entity(entity).despawn();
            continue;
        }
        glow.extend(burst_lights(impact.kind, impact.at, impact.age, impact.seed));
    }
}

/// Pulse the hit vignette: a fast rise, then a fade over the remainder.
pub fn animate_hit_vignettes(
    mut commands: Commands,
    time: Res<Time>,
    mut vignettes: Query<(Entity, &mut HitVignette, &mut BorderColor)>,
) {
    for (entity, mut v, mut border) in &mut vignettes {
        v.age += time.delta_secs();
        if v.age >= v.ttl {
            commands.entity(entity).despawn();
            continue;
        }
        let t = v.age / v.ttl;
        // Rise over the first 20%, ease back down over the rest.
        let shape = if t < 0.2 { t / 0.2 } else { (1.0 - t) / 0.8 };
        let alpha = shape.clamp(0.0, 1.0) * v.peak_alpha;
        *border = BorderColor::all(Color::srgba(0.9, 0.08, 0.08, alpha));
    }
}

/// Punch each damage numeral in, then float it upward and fade it out,
/// despawning when elapsed.
pub fn animate_damage_numerals(
    mut commands: Commands,
    time: Res<Time>,
    mut numerals: Query<(Entity, &mut DamageNumeral, &mut Node, &mut UiTransform, &Children)>,
    mut texts: Query<&mut TextColor>,
) {
    for (entity, mut numeral, mut node, mut transform, children) in &mut numerals {
        numeral.remaining -= time.delta_secs();
        if numeral.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let frac = (numeral.remaining / numeral.total).clamp(0.0, 1.0); // 1 → 0
        transform.scale = Vec2::splat(damage_numeral_scale(numeral.total - numeral.remaining));
        // Rise easing out: quick off the hit, settling as it fades.
        let risen = 1.0 - frac * frac;
        node.top = Val::Px(numeral.base_top - risen * numeral.rise);
        // Hold full opacity, then ease out over the final 50%.
        let alpha = (frac / 0.5).min(1.0);
        for child in children.iter() {
            if let Ok(mut tc) = texts.get_mut(child) {
                let c = tc.0.to_srgba();
                tc.0 = Color::srgba(c.red, c.green, c.blue, alpha);
            }
        }
    }
}

/// The light of a mana mote from `start` to `end`, `age` seconds in: an
/// orb flying its arc with a trail behind it, then a burst where it lands.
pub(crate) fn mote_lights(start: Vec3, end: Vec3, age: f32, colour: LinearRgba) -> Vec<Light> {
    let path = |t: f32| {
        let t = t.clamp(0.0, 1.0);
        start.lerp(end, t) + Vec3::Y * (t * PI).sin() * MANA_MOTE_LIFT
    };
    if age < MANA_MOTE_FLIGHT {
        let t = age / MANA_MOTE_FLIGHT;
        let reach = MANA_MOTE_TRAIL / MANA_MOTE_FLIGHT;
        let points = (0..8).map(|j| path(t - reach * j as f32 / 7.0)).collect();
        vec![
            Light::Trail { points, half: 0.13, colour },
            Light::Orb { at: path(t), radius: 0.26, colour: scaled(colour, 1.3) },
        ]
    } else {
        let k = ((age - MANA_MOTE_FLIGHT) / MANA_MOTE_POP).min(1.0);
        vec![Light::Orb { at: end, radius: 0.3 + 0.8 * k, colour: scaled(colour, 1.6 * (1.0 - k)) }]
    }
}

/// Fly each mana mote along its arc and burst it where it lands,
/// despawning it after.
pub fn animate_mana_motes(
    mut commands: Commands,
    time: Res<Time>,
    mut motes: Query<(Entity, &mut ManaMote)>,
    mut glow: ResMut<Glow>,
) {
    for (entity, mut mote) in &mut motes {
        mote.age += time.delta_secs();
        if mote.age >= MANA_MOTE_FLIGHT + MANA_MOTE_POP {
            commands.entity(entity).despawn();
            continue;
        }
        glow.extend(mote_lights(mote.start, mote.end, mote.age, mote.colour));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_damage_numeral_lands_big_and_snaps_to_size() {
        assert_eq!(damage_numeral_scale(0.0), DMG_NUMERAL_PUNCH);
        let midway = damage_numeral_scale(DMG_NUMERAL_PUNCH_SECS / 2.0);
        assert!(midway > 1.0 && midway < DMG_NUMERAL_PUNCH);
        assert_eq!(damage_numeral_scale(DMG_NUMERAL_PUNCH_SECS), 1.0);
        assert_eq!(damage_numeral_scale(DMG_NUMERAL_SECS), 1.0);
    }

    const AT: Vec3 = Vec3::new(2.0, 0.08, -3.0);

    #[test]
    fn every_burst_has_gone_out_by_the_end_of_its_life() {
        for kind in [Burst::Death, Burst::Spark, Burst::Dig] {
            assert!(!burst_lights(kind, AT, 0.0, 7).is_empty(), "{kind:?} starts lit");
            assert!(burst_lights(kind, AT, kind.ttl(), 7).is_empty(), "{kind:?} outlives its ttl");
        }
    }

    #[test]
    fn a_death_flashes_sends_a_wave_out_and_embers_up() {
        let at_start = burst_lights(Burst::Death, AT, 0.0, 3);
        assert!(matches!(at_start[0], Light::Pool { .. }), "the flash comes first");
        let wave = |age: f32| {
            burst_lights(Burst::Death, AT, age, 3).into_iter().find_map(|l| match l {
                Light::Wave { radius, .. } => Some(radius),
                _ => None,
            })
        };
        assert!(wave(0.1).unwrap() < wave(0.5).unwrap(), "the wave spreads");
        assert!(wave(DEATH_WAVE_SECS).is_none());
        // Every ember is lit at first, and all that are left have risen.
        let embers = |age: f32| {
            burst_lights(Burst::Death, AT, age, 3)
                .into_iter()
                .filter_map(|l| match l {
                    Light::Orb { at, .. } => Some(at),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(embers(0.0).len(), DEATH_EMBERS as usize);
        assert!(embers(0.5).iter().all(|e| e.y > AT.y + 0.4), "{:?}", embers(0.5));
    }

    #[test]
    fn sparks_fly_off_streaking_behind_them() {
        let trails: Vec<Vec<Vec3>> = burst_lights(Burst::Spark, AT, 0.1, 5)
            .into_iter()
            .filter_map(|l| match l {
                Light::Trail { points, .. } => Some(points),
                _ => None,
            })
            .collect();
        assert_eq!(trails.len(), SPARKS as usize);
        for points in trails {
            let (head, tail) = (points[0], points[1]);
            assert!(head.distance(AT) > tail.distance(AT), "the head leads");
            assert!(head.distance(AT) > 0.4);
        }
    }

    #[test]
    fn a_mana_mote_arcs_to_its_spell_and_bursts_there() {
        let (start, end) = (Vec3::new(-6.0, 0.3, 8.0), Vec3::new(4.0, 1.1, 0.0));
        let head = |age: f32| {
            mote_lights(start, end, age, LinearRgba::BLUE).into_iter().find_map(|l| match l {
                Light::Orb { at, .. } => Some(at),
                _ => None,
            })
        };
        assert!(head(0.0).unwrap().distance(start) < 1e-4);
        let midway = head(MANA_MOTE_FLIGHT / 2.0).unwrap();
        assert!(midway.y > end.y + 1.0, "it arcs over the table");
        // Its trail runs back along the way it came.
        let Some(Light::Trail { points, .. }) = mote_lights(start, end, MANA_MOTE_FLIGHT / 2.0, LinearRgba::BLUE).first().cloned()
        else {
            panic!("no trail")
        };
        assert!(points[0].distance(start) > points[7].distance(start));
        // Landed, it bursts on the spell.
        assert_eq!(head(MANA_MOTE_FLIGHT + 0.01), Some(end));
    }
}
