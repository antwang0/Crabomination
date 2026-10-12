//! Mana symbols in UI text: `{2}{W}{W}`, `{T}`, `{W/U}` drawn as pips.
//!
//! The engine writes every cost the Oracle way — `ManaCost::summary()`'s
//! `{3}{W}{W}`, an ability's `{2}{T}: Draw a card` — and the client printed
//! those strings as they came. [`ManaText`] draws one as text in which each
//! `{…}` is a round pip in the printed symbol's colour: a circle with a
//! letter or number on it, a hybrid split corner to corner.
//!
//! The UI font has no mana glyphs, so a pip is an image laid out *in* the
//! text (`InlineImage`, Bevy 0.20): it wraps with the words around it and
//! sits on their baseline. Until 0.20 a pip was a node, so mana text was a
//! flex row of word and pip nodes that could break only between words. Each
//! pip is rasterised once per symbol and pixel size ([`PipImages`]) from the
//! UI fonts Bevy has already loaded, at the size it is drawn, so it stays
//! crisp at any UI size.
//!
//! Put [`ManaText`] on a node (or spawn [`mana_text`]) wherever a string may
//! carry a cost; [`sync_mana_text`] rebuilds the text when the string or the
//! UI scale changes.

use ab_glyph::{Font as _, FontRef, PxScale, ScaleFont as _, point};
use bevy::asset::RenderAssetUsages;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::text::LineBreak;
use bevy::window::PrimaryWindow;
use crabomination::mana::Color as ManaColor;

use crate::theme::UiFonts;

/// One mana symbol, as far as drawing it goes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Pip {
    /// `{W}` … `{G}`.
    Colored(ManaColor),
    /// A number, or `{X}` / `{Y}` / `{Z}`: grey, with its text.
    Generic(String),
    /// `{C}`: colorless specifically.
    Colorless,
    /// `{W/U}`.
    Hybrid(ManaColor, ManaColor),
    /// `{2/W}`.
    MonoHybrid(u32, ManaColor),
    /// `{C/W}`.
    ColorlessHybrid(ManaColor),
    /// `{W/P}`: the color or 2 life.
    Phyrexian(ManaColor),
    /// `{W/U/P}`.
    PhyrexianHybrid(ManaColor, ManaColor),
    /// `{S}`.
    Snow,
    /// `{T}` and `{Q}`.
    Tap,
    Untap,
    /// `{E}`.
    Energy,
}

/// A run of a mana string: text as written (spaces and newlines kept), or
/// a pip.
#[derive(Clone, Debug, PartialEq)]
pub enum Run {
    Text(String),
    Pip(Pip),
}

fn color_of(letter: &str) -> Option<ManaColor> {
    Some(match letter {
        "W" => ManaColor::White,
        "U" => ManaColor::Blue,
        "B" => ManaColor::Black,
        "R" => ManaColor::Red,
        "G" => ManaColor::Green,
        _ => return None,
    })
}

/// The pip a `{…}` token names (without its braces), if it is one.
pub fn pip(token: &str) -> Option<Pip> {
    if let Some(c) = color_of(token) {
        return Some(Pip::Colored(c));
    }
    if !token.is_empty() && token.chars().all(|c| c.is_ascii_digit()) {
        return Some(Pip::Generic(token.to_string()));
    }
    match token {
        "X" | "Y" | "Z" => return Some(Pip::Generic(token.to_string())),
        "C" => return Some(Pip::Colorless),
        "S" => return Some(Pip::Snow),
        "T" => return Some(Pip::Tap),
        "Q" => return Some(Pip::Untap),
        "E" => return Some(Pip::Energy),
        _ => {}
    }
    let parts: Vec<&str> = token.split('/').collect();
    match parts.as_slice() {
        [a, "P"] => color_of(a).map(Pip::Phyrexian),
        [a, b, "P"] => Some(Pip::PhyrexianHybrid(color_of(a)?, color_of(b)?)),
        ["C", b] => color_of(b).map(Pip::ColorlessHybrid),
        [a, b] => match (a.parse::<u32>(), color_of(a), color_of(b)) {
            (Ok(n), _, Some(c)) => Some(Pip::MonoHybrid(n, c)),
            (_, Some(x), Some(y)) => Some(Pip::Hybrid(x, y)),
            _ => None,
        },
        _ => None,
    }
}

/// Split `text` into text and pips. A `{…}` that names no symbol stays
/// text, braces and all.
pub fn runs(text: &str) -> Vec<Run> {
    let mut out = Vec::new();
    let mut plain = String::new();
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        if c == '{'
            && let Some(end) = rest.find('}')
            && let Some(p) = pip(&rest[1..end])
        {
            if !plain.is_empty() {
                out.push(Run::Text(std::mem::take(&mut plain)));
            }
            out.push(Run::Pip(p));
            rest = &rest[end + 1..];
            continue;
        }
        plain.push(c);
        rest = &rest[c.len_utf8()..];
    }
    if !plain.is_empty() {
        out.push(Run::Text(plain));
    }
    out
}

/// Whether `text` has a mana symbol in it.
pub fn has_pips(text: &str) -> bool {
    runs(text).iter().any(|r| matches!(r, Run::Pip(_)))
}

/// A mana color's pip fill: the printed symbol's.
pub(crate) fn fill(c: ManaColor) -> Color {
    match c {
        ManaColor::White => Color::srgb(0.98, 0.96, 0.82),
        ManaColor::Blue => Color::srgb(0.68, 0.83, 0.95),
        ManaColor::Black => Color::srgb(0.76, 0.71, 0.68),
        ManaColor::Red => Color::srgb(0.93, 0.60, 0.50),
        ManaColor::Green => Color::srgb(0.60, 0.79, 0.64),
    }
}

pub(crate) const GREY: Color = Color::srgb(0.80, 0.78, 0.76);
pub(crate) const INK: Color = Color::srgb(0.07, 0.06, 0.06);

fn letter(c: ManaColor) -> &'static str {
    match c {
        ManaColor::White => "W",
        ManaColor::Blue => "U",
        ManaColor::Black => "B",
        ManaColor::Red => "R",
        ManaColor::Green => "G",
    }
}

/// A pip's two fills (the same twice for a single-color one) and the glyph
/// printed on it.
fn look(p: &Pip) -> (Color, Color, String) {
    match p {
        Pip::Colored(c) => (fill(*c), fill(*c), letter(*c).into()),
        Pip::Generic(n) => (GREY, GREY, n.clone()),
        Pip::Colorless => (GREY, GREY, "C".into()),
        Pip::Hybrid(a, b) => (fill(*a), fill(*b), String::new()),
        Pip::MonoHybrid(n, c) => (GREY, fill(*c), n.to_string()),
        Pip::ColorlessHybrid(c) => (GREY, fill(*c), String::new()),
        Pip::Phyrexian(c) => (fill(*c), fill(*c), "Φ".into()),
        Pip::PhyrexianHybrid(a, b) => (fill(*a), fill(*b), "Φ".into()),
        Pip::Snow => (Color::srgb(0.92, 0.95, 0.98), Color::srgb(0.92, 0.95, 0.98), "S".into()),
        Pip::Tap => (GREY, GREY, "↻".into()),
        Pip::Untap => (GREY, GREY, "⟲".into()),
        Pip::Energy => (Color::srgb(0.95, 0.80, 0.35), Color::srgb(0.95, 0.80, 0.35), "⚡".into()),
    }
}

/// A pip's diameter, in ems of the text around it. A pip sits on the
/// baseline (an inline box's bottom edge is), so at a capital's size it
/// reads as a letter of the line rather than floating over it.
const PIP_EM: f32 = 1.0;
/// Its glyph's font size (em), in pip diameters.
const GLYPH_PER_PIP: f32 = 0.69;
/// Clear space either side of a pip, in logical pixels, so a cost's pips
/// don't touch.
const PIP_GAP: f32 = 1.0;

/// Mana text on a node: the node is a `Text` whose spans and inline pips
/// [`sync_mana_text`] builds from `text`. `wrap` lets it wrap between words
/// (and pips); otherwise only a newline breaks a line.
#[derive(Component, Clone)]
pub struct ManaText {
    pub text: String,
    pub size: f32,
    pub color: Color,
    pub wrap: bool,
    /// What the children were last built for: everything above, and the
    /// UI scale.
    built: Option<(String, f32, Color, bool, f32)>,
}

impl ManaText {
    pub fn new(text: impl Into<String>, size: f32, color: Color) -> Self {
        Self { text: text.into(), size, color, wrap: false, built: None }
    }

    pub fn wrapping(mut self) -> Self {
        self.wrap = true;
        self
    }

    /// The text it draws.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Replace the text; it is rebuilt only if it differs. Through a `Mut`
    /// the call itself marks the component changed, so a caller that runs
    /// every frame compares with [`Self::text`] first.
    pub fn set(&mut self, text: &str) {
        if self.text != text {
            self.text = text.to_string();
        }
    }
}

/// A node carrying `text` as mana text: `(Node, ManaText)`, for a spawn
/// that has no other layout to give it.
pub fn mana_text(text: impl Into<String>, size: f32, color: Color) -> (Node, ManaText) {
    (Node::default(), ManaText::new(text, size, color))
}

/// Spawn `text` under `parent`: plain text, as it would have been, or —
/// when it carries a mana symbol — mana text that wraps between words.
pub fn spawn_text<'a>(
    parent: &'a mut ChildSpawnerCommands,
    fonts: &UiFonts,
    text: &str,
    size: f32,
    color: Color,
) -> EntityCommands<'a> {
    if has_pips(text) {
        parent.spawn((Node::default(), ManaText::new(text, size, color).wrapping()))
    } else {
        parent.spawn((Text::new(text), fonts.tf(size), TextColor(color)))
    }
}

/// Rasterised pips by symbol, diameter and gap in physical pixels.
#[derive(Resource, Default)]
pub struct PipImages(HashMap<(Pip, u32, u32), Handle<Image>>);

/// Bevy system: build each [`ManaText`] whose text, size, colour, wrap or
/// UI scale changed.
/// Waits for the UI fonts to load, since the pips' glyphs come from them.
pub fn sync_mana_text(
    mut commands: Commands,
    ui_fonts: Res<UiFonts>,
    fonts: Res<Assets<Font>>,
    mut images: ResMut<Assets<Image>>,
    mut pips: ResMut<PipImages>,
    window: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut texts: Query<(Entity, &mut ManaText)>,
) {
    let scale = window.single().map_or(1.0, Window::scale_factor) * ui_scale.0;
    let stale = |m: &ManaText| {
        m.built.as_ref().is_none_or(|(t, z, c, w, s)| {
            *t != m.text || *z != m.size || *c != m.color || *w != m.wrap || *s != scale
        })
    };
    if !texts.iter().any(|(_, m)| stale(m)) {
        return;
    }
    let faces: Option<Vec<FontRef<'_>>> = std::iter::once(&ui_fonts.sans)
        .chain(&ui_fonts.fallbacks)
        .map(|h| fonts.get(h).and_then(|f| FontRef::try_from_slice(f.data.data()).ok()))
        .collect();
    let Some(faces) = faces else { return };
    for (entity, mut mana) in &mut texts {
        if !stale(&mana) {
            continue;
        }
        mana.built = Some((mana.text.clone(), mana.size, mana.color, mana.wrap, scale));
        let font = ui_fonts.tf(mana.size);
        let color = TextColor(mana.color);
        let linebreak = if mana.wrap { LineBreak::WordBoundary } else { LineBreak::NoWrap };
        let diameter = (mana.size * PIP_EM * scale).round().max(1.0) as u32;
        let gap = (PIP_GAP * scale).round() as u32;
        let mut text = commands.entity(entity);
        text.despawn_children()
            .insert((Text::default(), font.clone(), color, TextLayout { linebreak, ..default() }));
        text.with_children(|t| {
            for run in runs(&mana.text) {
                match run {
                    Run::Text(s) => {
                        t.spawn((TextSpan::new(s), font.clone(), color));
                    }
                    Run::Pip(p) => {
                        let image = pips
                            .0
                            .entry((p.clone(), diameter, gap))
                            .or_insert_with(|| images.add(rasterize(&p, diameter, gap, &faces)))
                            .clone();
                        t.spawn(InlineImage {
                            image,
                            width: Some((diameter + 2 * gap) as f32 / scale),
                            height: Some(diameter as f32 / scale),
                            ..default()
                        });
                    }
                }
            }
        });
    }
}

/// One pip, `diameter` pixels across with `gap` clear pixels either side: a
/// disc (two-color ones split corner to corner) with its glyph centred on
/// it, from the first of `faces` that has every character of the glyph.
fn rasterize(pip: &Pip, diameter: u32, gap: u32, faces: &[FontRef<'_>]) -> Image {
    // 4×4 samples a pixel smooth the disc's edge and the split.
    const SAMPLES: u32 = 4;
    let (a, b, glyph) = look(pip);
    let rgb = |c: Color| {
        let c = c.to_srgba();
        [c.red, c.green, c.blue]
    };
    let (a, b, ink) = (rgb(a), rgb(b), rgb(INK));
    let (w, h) = (diameter + 2 * gap, diameter);
    let r = diameter as f32 / 2.0;
    let (cx, cy) = (gap as f32 + r, r);
    let mut px = vec![[0.0f32; 4]; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            for s in 0..SAMPLES * SAMPLES {
                let dx = x as f32 + ((s % SAMPLES) as f32 + 0.5) / SAMPLES as f32 - cx;
                let dy = y as f32 + ((s / SAMPLES) as f32 + 0.5) / SAMPLES as f32 - cy;
                if dx * dx + dy * dy <= r * r {
                    let c = if dx + dy < 0.0 { a } else { b };
                    for i in 0..3 {
                        acc[i] += c[i];
                    }
                    acc[3] += 1.0;
                }
            }
            if acc[3] > 0.0 {
                let n = (SAMPLES * SAMPLES) as f32;
                px[(y * w + x) as usize] = [acc[0] / acc[3], acc[1] / acc[3], acc[2] / acc[3], acc[3] / n];
            }
        }
    }
    if let Some(face) = faces.iter().find(|f| glyph.chars().all(|c| f.glyph_id(c).0 != 0)) {
        // `PxScale` is the font's line height, not its em; Bevy sizes text by
        // the em, so convert with this face's own ratio or a font with tall
        // line metrics (the math fallback's Φ) draws small.
        let em_to_px = face.height_unscaled() / face.units_per_em().unwrap_or(1000.0);
        let mut size = diameter as f32 * GLYPH_PER_PIP * em_to_px;
        let outlines = loop {
            let scaled = face.as_scaled(PxScale::from(size));
            let mut x = 0.0;
            let outlines: Vec<_> = glyph
                .chars()
                .filter_map(|c| {
                    let id = face.glyph_id(c);
                    let g = id.with_scale_and_position(size, point(x, 0.0));
                    x += scaled.h_advance(id);
                    face.outline_glyph(g)
                })
                .collect();
            let width = outlines.iter().map(|o| o.px_bounds().max.x).fold(f32::MIN, f32::max)
                - outlines.iter().map(|o| o.px_bounds().min.x).fold(f32::MAX, f32::min);
            // "10" and up: shrink until it fits inside the disc.
            if width <= diameter as f32 * 0.8 || size < 4.0 {
                break outlines;
            }
            size *= 0.9;
        };
        if let (Some(min_x), Some(min_y), Some(max_x), Some(max_y)) = (
            outlines.iter().map(|o| o.px_bounds().min.x).reduce(f32::min),
            outlines.iter().map(|o| o.px_bounds().min.y).reduce(f32::min),
            outlines.iter().map(|o| o.px_bounds().max.x).reduce(f32::max),
            outlines.iter().map(|o| o.px_bounds().max.y).reduce(f32::max),
        ) {
            // Whole-pixel offsets keep the glyph's stems on the pixel grid.
            let ox = (cx - (min_x + max_x) / 2.0).round();
            let oy = (cy - (min_y + max_y) / 2.0).round();
            for o in &outlines {
                let bounds = o.px_bounds();
                o.draw(|gx, gy, coverage| {
                    let x = (bounds.min.x + gx as f32 + ox) as i32;
                    let y = (bounds.min.y + gy as f32 + oy) as i32;
                    if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                        return;
                    }
                    let p = &mut px[(y as u32 * w + x as u32) as usize];
                    for i in 0..3 {
                        p[i] += (ink[i] - p[i]) * coverage;
                    }
                });
            }
        }
    }
    let data = px.iter().flat_map(|p| p.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)).collect();
    Image::new(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        // The main world keeps it: `InlineImage` sizes its box from it.
        RenderAssetUsages::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cost_splits_into_pips_and_text() {
        use Run::{Pip as P, Text as T};
        assert_eq!(
            runs("{2}{W}{W}"),
            vec![P(Pip::Generic("2".into())), P(Pip::Colored(ManaColor::White)), P(Pip::Colored(ManaColor::White))]
        );
        assert_eq!(runs("{2}{T}: Draw a card"), vec![P(Pip::Generic("2".into())), P(Pip::Tap), T(": Draw a card".into())]);
        // Spaces and newlines stay in the text, where the layout reads them.
        assert_eq!(
            runs("Pay {X} (+2 tax)\nor not"),
            vec![T("Pay ".into()), P(Pip::Generic("X".into())), T(" (+2 tax)\nor not".into())]
        );
    }

    #[test]
    fn every_kind_of_symbol_the_engine_writes_is_a_pip() {
        assert_eq!(pip("W/U"), Some(Pip::Hybrid(ManaColor::White, ManaColor::Blue)));
        assert_eq!(pip("2/R"), Some(Pip::MonoHybrid(2, ManaColor::Red)));
        assert_eq!(pip("B/P"), Some(Pip::Phyrexian(ManaColor::Black)));
        assert_eq!(pip("R/G/P"), Some(Pip::PhyrexianHybrid(ManaColor::Red, ManaColor::Green)));
        assert_eq!(pip("C/W"), Some(Pip::ColorlessHybrid(ManaColor::White)));
        assert_eq!(pip("C"), Some(Pip::Colorless));
        assert_eq!(pip("S"), Some(Pip::Snow));
        assert_eq!(pip("10"), Some(Pip::Generic("10".into())));
        // Everything `ManaCost::summary()` writes parses back to one pip each.
        use crabomination::mana::{ManaCost, ManaSymbol};
        let cost = ManaCost::new(vec![
            ManaSymbol::X,
            ManaSymbol::Generic(3),
            ManaSymbol::Colored(ManaColor::Green),
            ManaSymbol::Hybrid(ManaColor::White, ManaColor::Black),
            ManaSymbol::Phyrexian(ManaColor::Red),
            ManaSymbol::MonoHybrid(2, ManaColor::Blue),
            ManaSymbol::Colorless(1),
            ManaSymbol::Snow,
        ]);
        let pips = runs(&cost.summary()).iter().filter(|r| matches!(r, Run::Pip(_))).count();
        assert_eq!(pips, cost.symbols.len(), "{}", cost.summary());
    }

    #[test]
    fn braces_that_name_no_symbol_stay_text() {
        assert_eq!(runs("{oops}"), vec![Run::Text("{oops}".into())]);
        assert!(!has_pips("Draw a card"));
        assert!(has_pips("Kicker {1}{R}"));
    }

    /// The UI font and its fallbacks, as the client loads them.
    fn ui_font_bytes() -> Vec<Vec<u8>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        std::iter::once(crate::theme::FONT_PATH)
            .chain(crate::theme::FALLBACK_FONT_PATHS)
            .map(|p| std::fs::read(root.join(p)).expect(p))
            .collect()
    }

    #[test]
    fn a_pip_is_a_disc_with_its_glyph_inked_on_it() {
        let bytes = ui_font_bytes();
        let faces: Vec<FontRef<'_>> = bytes.iter().map(|b| FontRef::try_from_slice(b).unwrap()).collect();
        let rgba = |img: &Image, x: u32, y: u32| {
            let i = ((y * img.width() + x) * 4) as usize;
            img.data.as_ref().unwrap()[i..i + 4].to_vec()
        };
        let white = rasterize(&Pip::Colored(ManaColor::White), 40, 2, &faces);
        assert_eq!((white.width(), white.height()), (44, 40));
        assert_eq!(rgba(&white, 0, 0)[3], 0, "outside the disc is clear");
        // Just inside the left edge: the white fill, opaque.
        let white_fill = fill(ManaColor::White).to_srgba().to_u8_array();
        assert_eq!(rgba(&white, 5, 20), white_fill.to_vec());
        // A hybrid is split corner to corner, first colour top-left.
        let hybrid = rasterize(&Pip::Hybrid(ManaColor::Red, ManaColor::Green), 40, 2, &faces);
        assert_eq!(rgba(&hybrid, 12, 10)[..3], fill(ManaColor::Red).to_srgba().to_u8_array()[..3]);
        assert_eq!(rgba(&hybrid, 32, 30)[..3], fill(ManaColor::Green).to_srgba().to_u8_array()[..3]);
        // Every symbol's glyph is in one of the fonts, so none is left blank.
        for p in ["W", "T", "Q", "E", "B/P", "S", "C", "10", "X"] {
            let img = rasterize(&pip(p).unwrap(), 40, 2, &faces);
            let inked = (0..40).flat_map(|y| (0..44).map(move |x| (x, y))).any(|(x, y)| rgba(&img, x, y)[1] < 64);
            assert!(inked, "{{{p}}} has no glyph");
        }
    }

    /// "Pay {2}" is one `Text`: the words a span, the pip an inline image
    /// sized in logical pixels so it is drawn at the pixels it was made at.
    #[test]
    fn mana_text_is_one_text_with_its_pips_inline() {
        let mut app = App::new();
        app.init_resource::<Assets<Font>>()
            .init_resource::<Assets<Image>>()
            .init_resource::<PipImages>()
            .insert_resource(UiScale(2.0))
            .add_systems(Update, sync_mana_text);
        let mut fonts = app.world_mut().resource_mut::<Assets<Font>>();
        let mut handles: Vec<Handle<Font>> = ui_font_bytes().into_iter().map(|b| fonts.add(Font::from_bytes(b))).collect();
        let sans = handles.remove(0);
        app.insert_resource(UiFonts { sans, fallbacks: handles });
        let text = app.world_mut().spawn(mana_text("Pay {2}", 13.0, Color::WHITE)).id();
        app.update();
        let world = app.world();
        assert!(world.get::<Text>(text).is_some());
        let children: Vec<Entity> = world.get::<Children>(text).unwrap().iter().collect();
        assert_eq!(world.get::<TextSpan>(children[0]).map(|s| s.0.as_str()), Some("Pay "));
        let image = world.get::<InlineImage>(children[1]).expect("the pip is inline");
        // 13 px at UI scale 2 is a 26 px disc, plus 2 px a side.
        assert_eq!((image.width, image.height), (Some(15.0), Some(13.0)));
        assert_eq!(world.resource::<Assets<Image>>().get(&image.image).unwrap().width(), 30);
    }
}
