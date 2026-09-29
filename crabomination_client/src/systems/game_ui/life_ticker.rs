//! Life totals that count to their new value, and a numeral beside the
//! seat's HUD row for each change.
//!
//! Every life readout — the viewer's badge, each opponent's row, a pod's
//! seat plates — jumped straight to the new total, so a burn spell or a
//! lifelink swing to an opponent read as a number that was simply different.
//! A readout now counts from the old total to the new one over most of a
//! second, lit red for a loss or green for a gain and swelling as it starts,
//! and a numeral in the style of the creature damage numerals (outlined,
//! landing big) rises beside the seat's row. The numerals were a thin bare
//! "-5" at fixed offsets from the screen corners, which the HUD's rows had
//! since moved out from under: they printed over the hand and deck chips.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};

use super::PlayerHudPanel;
use crate::net_plugin::CurrentView;
use crate::theme::UiFonts;

/// Seconds a readout takes to count to its new total.
const TICK_SECS: f32 = 0.75;
/// How much a readout swells as its count starts.
const TICK_SWELL: f32 = 0.35;
/// Gap (px) between a seat's HUD panel and its numeral.
const NUMERAL_GAP: f32 = 10.0;
/// How far (px) a life numeral rises, through its row's level: a creature's
/// rises further, but the HUD rows sit at the top of the screen.
const NUMERAL_RISE: f32 = 18.0;
/// A gain's numeral: a green that holds up inside the dark outline.
const GAIN_GREEN: Color = Color::srgb(0.36, 0.95, 0.46);

/// A seat's count from one total to another.
#[derive(Clone, Copy, Debug)]
struct Tick {
    from: i32,
    to: i32,
    age: f32,
}

impl Tick {
    /// How far through the count (0-1).
    fn progress(&self) -> f32 {
        (self.age / TICK_SECS).clamp(0.0, 1.0)
    }

    /// The total to show: easing out, so a big swing races off and settles.
    fn shown(&self) -> i32 {
        let t = 1.0 - (1.0 - self.progress()).powi(3);
        self.from + ((self.to - self.from) as f32 * t).round() as i32
    }
}

/// Each seat's last life total and, while it counts, its [`Tick`].
#[derive(Resource, Default)]
pub struct LifeTicker {
    last: HashMap<usize, i32>,
    ticks: HashMap<usize, Tick>,
    primed: bool,
}

impl LifeTicker {
    /// The total to show for `seat`, whose life is `life`: part-way through
    /// the count while one plays out.
    pub fn shown(&self, seat: usize, life: i32) -> i32 {
        self.ticks.get(&seat).map_or(life, Tick::shown)
    }

    /// Note each seat's life total from a view, starting a count for every
    /// seat whose total moved; returns those seats and how far each moved.
    /// The first view only primes it, so joining a game mid-way (or the
    /// first view of a match) counts nothing.
    fn observe(&mut self, totals: impl IntoIterator<Item = (usize, i32)>) -> Vec<(usize, i32)> {
        let mut moved = Vec::new();
        for (seat, life) in totals {
            let before = self.last.insert(seat, life);
            let Some(before) = before.filter(|b| *b != life && self.primed) else { continue };
            // A change landing mid-count carries on from what is showing.
            let from = self.ticks.get(&seat).map_or(before, Tick::shown);
            self.ticks.insert(seat, Tick { from, to: life, age: 0.0 });
            moved.push((seat, life - before));
        }
        self.primed = true;
        moved
    }

    fn advance(&mut self, dt: f32) {
        for tick in self.ticks.values_mut() {
            tick.age += dt;
        }
        self.ticks.retain(|_, t| t.age < TICK_SECS);
    }
}

/// How a life readout words its total.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LifeStyle {
    /// The viewer's badge: "♥ 17 (−3)".
    Badge,
    /// An opponent's row: "♥ 35 −5".
    Row,
}

/// A text showing `seat`'s life total, re-worded as the total counts.
#[derive(Component, Clone, Copy, Debug)]
pub struct LifeReadout {
    pub seat: usize,
    pub starting: i32,
    pub style: LifeStyle,
    /// Its colour when not counting.
    pub ink: Color,
}

impl LifeReadout {
    pub fn label(&self, life: i32) -> String {
        match self.style {
            LifeStyle::Badge => super::player_stats::badge_life_label(life, self.starting),
            LifeStyle::Row => super::table_awareness::big_life_label(life, self.starting),
        }
    }
}

/// A life numeral, kept beside its seat's HUD row as it rises.
#[derive(Component)]
pub struct LifeNumeral {
    seat: usize,
}

/// On a new view, start a count for each seat whose life moved and float a
/// numeral beside its HUD row. Runs before the HUD rows are rebuilt, so they
/// are built showing the count's first frame rather than the new total.
pub fn track_life_changes(
    mut commands: Commands,
    view: Res<CurrentView>,
    fonts: Res<UiFonts>,
    mut ticker: ResMut<LifeTicker>,
) {
    if !view.is_changed() {
        return;
    }
    let Some(cv) = &view.0 else { return };
    for (seat, delta) in ticker.observe(cv.players.iter().map(|p| (p.seat, p.life))) {
        let (text, colour) =
            if delta < 0 { (format!("{delta}"), crate::systems::impact::DMG_NUMERAL_RED) } else { (format!("+{delta}"), GAIN_GREEN) };
        // Parked off screen; `place_life_numerals` puts it beside the row.
        let numeral = crate::systems::impact::spawn_numeral(&mut commands, &fonts, text, colour, Vec2::splat(-1000.0));
        commands.entity(numeral).insert(LifeNumeral { seat });
    }
}

/// Count each ticking readout toward its total: re-word it, tint it red for
/// a loss or green for a gain, and swell it as the count starts.
pub fn tick_life_readouts(
    time: Res<Time>,
    view: Res<CurrentView>,
    mut ticker: ResMut<LifeTicker>,
    mut readouts: Query<(&LifeReadout, &mut Text, &mut TextColor, &mut UiTransform)>,
) {
    ticker.advance(crate::systems::animate::anim_dt(&time));
    let Some(cv) = &view.0 else { return };
    for (readout, mut text, mut ink, mut transform) in &mut readouts {
        let Some(life) = cv.players.iter().find(|p| p.seat == readout.seat).map(|p| p.life) else { continue };
        let (shown, colour, swell) = match ticker.ticks.get(&readout.seat) {
            Some(tick) => {
                let left = 1.0 - tick.progress();
                let lit = if tick.to < tick.from { crate::systems::impact::DMG_NUMERAL_RED } else { GAIN_GREEN };
                (tick.shown(), lit.mix(&readout.ink, 1.0 - left), 1.0 + TICK_SWELL * left * left)
            }
            None => (life, readout.ink, 1.0),
        };
        let label = readout.label(shown);
        if text.0 != label {
            text.0 = label;
        }
        if ink.0 != colour {
            ink.0 = colour;
        }
        if transform.scale.x != swell {
            transform.scale = Vec2::splat(swell);
        }
    }
}

/// Keep each life numeral by its seat. In a pod it rises off the seat's
/// name plate on the table (`table_tint`), which shows the seat's life:
/// beside the HUD panels, a small window's top-left panel ran into the
/// top-right one and their numerals overlapped. In a duel, with no plates,
/// it sits just outside the seat's HUD panel, level with the seat's life
/// readout: to the right of a panel on the left of the screen, to the left
/// of one on the right. The rows are rebuilt on every view, so the numeral
/// finds them afresh each frame. `DamageNumeral` does the rest (the punch,
/// the rise, the fade).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn place_life_numerals(
    view: Res<CurrentView>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    panels: Query<(&PlayerHudPanel, &UiGlobalTransform, &ComputedNode)>,
    readouts: Query<(&LifeReadout, &UiGlobalTransform, &ComputedNode)>,
    plates: Query<(&crate::systems::table_tint::SeatNamePlate, &UiGlobalTransform, &ComputedNode, &Node), Without<LifeNumeral>>,
    mut numerals: Query<(&LifeNumeral, &mut Node, &mut UiTransform, &mut crate::systems::impact::DamageNumeral)>,
) {
    let Some(cv) = &view.0 else { return };
    let Ok(window) = windows.single() else { return };
    let mid = window.width() / ui_scale.0 / 2.0;
    // A node's rect in UI px.
    let rect = |t: &UiGlobalTransform, c: &ComputedNode| {
        Rect::from_center_size(t.translation * c.inverse_scale_factor(), c.size() * c.inverse_scale_factor())
    };
    for (numeral, mut node, mut transform, mut motion) in &mut numerals {
        let plate = plates
            .iter()
            .find(|(p, .., n)| p.0 == numeral.seat && n.display != Display::None)
            .map(|(_, t, c, _)| rect(t, c));
        if let Some(plate) = plate {
            let hud: Vec<Rect> = panels.iter().map(|(_, t, c)| rect(t, c)).collect();
            node.left = Val::Px(plate.center().x);
            transform.translation = Val2::percent(-50.0, -50.0);
            motion.rebase(plate_numeral_level(plate, &hud), NUMERAL_RISE * 2.0);
            continue;
        }
        // The viewer's panel carries `usize::MAX` until
        // `sync_player_hud_seat` gives it the viewer's seat.
        let theirs = |p: &PlayerHudPanel| {
            p.seat == numeral.seat || (numeral.seat == cv.your_seat && p.seat == usize::MAX)
        };
        let Some(panel) = panels.iter().find(|(p, ..)| theirs(p)).map(|(_, t, c)| rect(t, c)) else {
            continue;
        };
        let level = readouts
            .iter()
            .find(|(r, ..)| r.seat == numeral.seat)
            .map_or(panel.center().y, |(_, t, c)| rect(t, c).center().y);
        let (x, pull) = if panel.center().x < mid {
            (panel.max.x + NUMERAL_GAP, 0.0)
        } else {
            (panel.min.x - NUMERAL_GAP, -100.0)
        };
        node.left = Val::Px(x);
        transform.translation = Val2::percent(pull, -50.0);
        motion.rebase(level, NUMERAL_RISE);
    }
}

/// Half a life numeral's height and width (UI px), near enough.
const NUMERAL_HALF: Vec2 = Vec2::new(40.0, 20.0);

/// The level (UI px) a pod's life numeral rises through by its seat plate
/// `plate`: above the plate, or below it where the rise would run into a
/// HUD panel (`hud`) — a small window's top-right panel comes down to the
/// far seats' plates.
fn plate_numeral_level(plate: Rect, hud: &[Rect]) -> f32 {
    let travel = NUMERAL_RISE * 2.0;
    let above = plate.min.y - NUMERAL_GAP - NUMERAL_HALF.y - NUMERAL_RISE;
    let path = Rect::from_center_size(
        Vec2::new(plate.center().x, above),
        Vec2::new(NUMERAL_HALF.x * 2.0, travel + NUMERAL_HALF.y * 2.0),
    );
    if hud.iter().all(|r| r.intersect(path).is_empty()) {
        above
    } else {
        plate.max.y + NUMERAL_GAP + NUMERAL_HALF.y + NUMERAL_RISE
    }
}

/// A new match starts with nothing counting and nothing remembered.
pub fn reset_life_ticker(mut ticker: ResMut<LifeTicker>) {
    *ticker = LifeTicker::default();
}

/// Build a readout's text for `seat`'s life (`life`, counting per
/// `ticker`), tagged so it counts.
pub(super) fn readout_text(
    ticker: &LifeTicker,
    readout: LifeReadout,
    life: i32,
    fonts: &UiFonts,
    size: f32,
) -> impl Bundle {
    (
        Text::new(readout.label(ticker.shown(readout.seat, life))),
        fonts.tf(size),
        TextColor(readout.ink),
        readout,
        Pickable::IGNORE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_total_counts_to_its_new_value() {
        let mut ticker = LifeTicker::default();
        // The first view primes: nothing counts.
        assert!(ticker.observe([(0, 20), (1, 20)]).is_empty());
        assert_eq!(ticker.shown(1, 20), 20);
        // Seat 1 takes 5: it counts down from 20...
        assert_eq!(ticker.observe([(0, 20), (1, 15)]), vec![(1, -5)]);
        assert_eq!(ticker.shown(1, 15), 20);
        ticker.advance(TICK_SECS * 0.3);
        let partway = ticker.shown(1, 15);
        assert!(partway < 20 && partway > 15, "{partway}");
        // ...and lands on 15.
        ticker.advance(TICK_SECS);
        assert_eq!(ticker.shown(1, 15), 15);
        assert!(ticker.ticks.is_empty());
    }

    #[test]
    fn a_plate_numeral_rises_above_its_plate_unless_the_hud_is_there() {
        let plate = Rect::new(500.0, 300.0, 600.0, 320.0);
        // Open table above: it rises clear of the plate's top.
        let open = plate_numeral_level(plate, &[Rect::new(0.0, 0.0, 400.0, 100.0)]);
        assert!(open + NUMERAL_RISE + NUMERAL_HALF.y <= plate.min.y, "{open}");
        // The HUD panel comes down to the plate: it rises to below it.
        let crowded = plate_numeral_level(plate, &[Rect::new(450.0, 0.0, 900.0, 280.0)]);
        assert!(crowded - NUMERAL_RISE - NUMERAL_HALF.y >= plate.max.y, "{crowded}");
    }

    #[test]
    fn a_change_mid_count_carries_on_from_what_shows() {
        let mut ticker = LifeTicker::default();
        ticker.observe([(0, 20)]);
        ticker.observe([(0, 10)]);
        ticker.advance(TICK_SECS * 0.3);
        let showing = ticker.shown(0, 10);
        // A lifelink gain lands before the loss has finished counting.
        assert_eq!(ticker.observe([(0, 13)]), vec![(0, 3)]);
        assert_eq!(ticker.shown(0, 13), showing);
        ticker.advance(TICK_SECS);
        assert_eq!(ticker.shown(0, 13), 13);
    }
}
