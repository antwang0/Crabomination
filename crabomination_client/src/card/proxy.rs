//! Drawn card faces for cards with no art: tokens, and anything Scryfall has
//! no image for.
//!
//! A card without art used to be a white rectangle with its name in thin
//! grey text, centred — at the table's ~125 px a card, the strokes averaged
//! away in the mips and a pile of different tokens read as a pile of blanks.
//! A proxy face is laid out like a card instead: a frame in the card's
//! colours, a dark name bar, a big initial in the art box (so "Goblin" and
//! "Treasure" differ at a glance), a type line, the keywords in the text box
//! and a P/T box where a printed card has one — which is where the P/T badge
//! (`pt_label`) sits, so the live P/T covers it.
//!
//! **Tokens get their facts into the face.** A token has no catalog card to
//! read, but its permanent view does: name, colours, types, keywords, base
//! P/T. [`proxy_asset_path`] encodes a [`ProxyFace`] into a `cards/proxy_…`
//! asset path, and the placeholder asset reader
//! (`scryfall::CardPlaceholderReader`) — which already draws any missing
//! `cards/…` image — decodes it and draws the face. Going through a path
//! keeps every consumer of card art working unchanged: the 3-D face, the
//! hover preview and the Alt popup load it like any card image, the asset
//! server shares one texture between identical tokens, and it gets its mips
//! like any card image. A token whose real art is on disk keeps its art.

use ab_glyph::{FontVec, PxScale};
use crabomination::mana::Color;
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut, text_size};
use imageproc::rect::Rect;

/// The face size: the 63:88 card proportion at a size whose mips a table
/// card samples well (card art is 745×1040; the table never needs mip 0).
pub const PROXY_W: u32 = 488;
pub const PROXY_H: u32 = 680;

/// What a drawn face shows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProxyFace {
    pub name: String,
    /// Empty = colourless.
    pub colors: Vec<Color>,
    /// "Token Creature — Goblin"; empty when unknown.
    pub type_line: String,
    /// Keyword names for the text box ("Flying", "Haste").
    pub keywords: Vec<String>,
    /// Base power / toughness, for a creature.
    pub pt: Option<(i32, i32)>,
}

impl ProxyFace {
    /// A face that knows only the card's name (an art-less card whose facts
    /// the asset reader doesn't have).
    pub fn named(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Default::default() }
    }
}

/// File-name prefix of an encoded proxy face under `cards/`.
const PREFIX: &str = "proxy_";
/// The longest encoded face; a path component past 255 bytes fails to open
/// with an error that isn't "not found", which the reader would not draw.
const MAX_ENCODED: usize = 220;

/// The `cards/…` asset path that draws `face`. Deterministic, so identical
/// tokens share one texture.
pub fn proxy_asset_path(face: &ProxyFace) -> String {
    let colors: String = face.colors.iter().map(|c| color_letter(*c)).collect();
    let pt = face.pt.map(|(p, t)| format!("{p}/{t}")).unwrap_or_default();
    let mut keywords = face.keywords.join(",");
    let mut spec;
    loop {
        spec = format!("{}|{}|{}|{}|{}", face.name, colors, face.type_line, keywords, pt);
        if escape(&spec).len() <= MAX_ENCODED || keywords.is_empty() {
            break;
        }
        // Too long for a file name: drop keywords from the end.
        keywords = keywords.rsplit_once(',').map(|(head, _)| head.to_string()).unwrap_or_default();
    }
    format!("cards/{PREFIX}{}.png", escape(&spec))
}

/// The face a `cards/proxy_….png` file stem encodes, if it is one.
pub fn parse_proxy_stem(stem: &str) -> Option<ProxyFace> {
    let spec = unescape(stem.strip_prefix(PREFIX)?)?;
    let mut parts = spec.splitn(5, '|');
    let name = parts.next()?.to_string();
    let colors = parts.next()?.chars().filter_map(letter_color).collect();
    let type_line = parts.next()?.to_string();
    let keywords = parts.next()?.split(',').filter(|k| !k.is_empty()).map(str::to_string).collect();
    let pt = parts.next()?.split_once('/').and_then(|(p, t)| Some((p.parse().ok()?, t.parse().ok()?)));
    Some(ProxyFace { name, colors, type_line, keywords, pt })
}

/// Alphanumerics as they are, every other byte as `_XX`: safe in a file name
/// on every platform and in a Bevy asset path (no `#`, `:` or separators).
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() {
            out.push(b as char);
        } else {
            out.push_str(&format!("_{b:02X}"));
        }
    }
    out
}

fn unescape(s: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(s.len());
    let mut it = s.bytes();
    while let Some(b) = it.next() {
        if b == b'_' {
            let hex = [it.next()?, it.next()?];
            bytes.push(u8::from_str_radix(std::str::from_utf8(&hex).ok()?, 16).ok()?);
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8(bytes).ok()
}

fn color_letter(c: Color) -> char {
    match c {
        Color::White => 'W',
        Color::Blue => 'U',
        Color::Black => 'B',
        Color::Red => 'R',
        Color::Green => 'G',
    }
}

fn letter_color(c: char) -> Option<Color> {
    Some(match c {
        'W' => Color::White,
        'U' => Color::Blue,
        'B' => Color::Black,
        'R' => Color::Red,
        'G' => Color::Green,
        _ => return None,
    })
}

/// The frame's fill: the card's colour, gold for two or more, grey for none.
fn frame_colour(colors: &[Color]) -> [u8; 3] {
    match colors {
        [] => [150, 150, 146],
        [Color::White] => [236, 228, 200],
        [Color::Blue] => [70, 120, 185],
        [Color::Black] => [72, 66, 70],
        [Color::Red] => [196, 78, 56],
        [Color::Green] => [64, 132, 80],
        _ => [205, 172, 82],
    }
}

fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> Rgba<u8> {
    let c = |i: usize| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t).round() as u8;
    Rgba([c(0), c(1), c(2), 255])
}

/// Draw `text` with strokes thickened by `weight` px — the only face on
/// disk is a Light weight, which minifies to nothing at table size.
#[allow(clippy::too_many_arguments)]
fn bold_text(img: &mut RgbaImage, ink: Rgba<u8>, x: i32, y: i32, scale: PxScale, font: &FontVec, text: &str, weight: i32) {
    for dx in 0..=weight {
        for dy in 0..=weight / 2 {
            draw_text_mut(img, ink, x + dx, y + dy, scale, font, text);
        }
    }
}

/// Shrink `scale` until `text` fits `max_w` px.
fn fit(font: &FontVec, text: &str, mut scale: f32, max_w: u32) -> PxScale {
    while scale > 12.0 && text_size(PxScale::from(scale), font, text).0 > max_w {
        scale -= 2.0;
    }
    PxScale::from(scale)
}

/// Greedy word-wrap of `text` into lines at most `max_w` px wide.
fn wrap(font: &FontVec, scale: PxScale, text: &str, max_w: u32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let trial = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if text_size(scale, font, &trial).0 > max_w && !line.is_empty() {
            lines.push(std::mem::replace(&mut line, word.to_string()));
        } else {
            line = trial;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Draw `face` as a card. With no font, the frame alone.
pub fn render_proxy(face: &ProxyFace, font: Option<&FontVec>) -> RgbaImage {
    let (w, h) = (PROXY_W, PROXY_H);
    let fill = frame_colour(&face.colors);
    let dark = [18, 18, 20];
    let light = [246, 242, 232];
    // Black card border, then the coloured frame.
    let mut img = RgbaImage::from_pixel(w, h, Rgba([12, 12, 14, 255]));
    let border = 16u32;
    draw_filled_rect_mut(&mut img, Rect::at(border as i32, border as i32).of_size(w - 2 * border, h - 2 * border), mix(fill, fill, 0.0));

    let inset = 30i32;
    let inner_w = w - 2 * inset as u32;
    // Name bar: dark, so the name reads light on it at any frame colour.
    let (name_y, name_h) = (28i32, 58u32);
    draw_filled_rect_mut(&mut img, Rect::at(inset, name_y).of_size(inner_w, name_h), mix(dark, fill, 0.15));
    // Art box: the frame colour, light at the top to dark at the bottom.
    let (art_y, art_h) = (name_y + name_h as i32 + 10, 292u32);
    for row in 0..art_h {
        let t = row as f32 / art_h as f32;
        let colour = mix(mix_rgb(fill, light, 0.45), mix_rgb(fill, dark, 0.55), t);
        draw_filled_rect_mut(&mut img, Rect::at(inset, art_y + row as i32).of_size(inner_w, 1), colour);
    }
    // Type line and text box.
    let (type_y, type_h) = (art_y + art_h as i32 + 8, 42u32);
    draw_filled_rect_mut(&mut img, Rect::at(inset, type_y).of_size(inner_w, type_h), mix(dark, fill, 0.15));
    let (text_y, text_h) = (type_y + type_h as i32 + 8, h as i32 - (type_y + type_h as i32 + 8) - 44);
    draw_filled_rect_mut(&mut img, Rect::at(inset, text_y).of_size(inner_w, text_h as u32), mix(light, fill, 0.12));
    // P/T box over the text box's lower right, where a printed card has it.
    let pt_box = face.pt.map(|_| {
        let (bw, bh) = (104u32, 56u32);
        let rect = Rect::at(w as i32 - inset - bw as i32 + 8, h as i32 - 36 - bh as i32).of_size(bw, bh);
        draw_filled_rect_mut(&mut img, rect, mix(dark, fill, 0.15));
        rect
    });

    let Some(font) = font else { return img };
    let ink_light = Rgba([250, 248, 240, 255]);
    let ink_dark = Rgba([20, 20, 22, 255]);

    // The name.
    let name_scale = fit(font, &face.name, 40.0, inner_w - 24);
    let (_, name_th) = text_size(name_scale, font, &face.name);
    bold_text(&mut img, ink_light, inset + 12, name_y + (name_h as i32 - name_th as i32) / 2 - 4, name_scale, font, &face.name, 2);

    // A big initial in the art box: tokens of different kinds differ at a
    // glance, before the name is read.
    if let Some(initial) = face.name.chars().find(|c| c.is_alphanumeric()) {
        let initial = initial.to_uppercase().to_string();
        let scale = PxScale::from(230.0);
        let (iw, ih) = text_size(scale, font, &initial);
        let ink = mix(fill, light, 0.75);
        bold_text(
            &mut img,
            ink,
            inset + (inner_w as i32 - iw as i32) / 2,
            art_y + (art_h as i32 - ih as i32) / 2 - 20,
            scale,
            font,
            &initial,
            6,
        );
    }

    // Type line.
    if !face.type_line.is_empty() {
        let scale = fit(font, &face.type_line, 26.0, inner_w - 24);
        let (_, th) = text_size(scale, font, &face.type_line);
        bold_text(&mut img, ink_light, inset + 12, type_y + (type_h as i32 - th as i32) / 2 - 2, scale, font, &face.type_line, 1);
    }

    // Keywords in the text box, wrapped, clear of the P/T box.
    if !face.keywords.is_empty() {
        let scale = PxScale::from(30.0);
        let text = face.keywords.join(", ");
        let right = pt_box.map(|_| 110).unwrap_or(0);
        let mut y = text_y + 14;
        for line in wrap(font, scale, &text, inner_w - 28 - right).into_iter().take(3) {
            bold_text(&mut img, ink_dark, inset + 14, y, scale, font, &line, 1);
            y += 40;
        }
    }

    // P/T.
    if let (Some((p, t)), Some(rect)) = (face.pt, pt_box) {
        let text = format!("{p}/{t}");
        let scale = fit(font, &text, 40.0, rect.width() - 16);
        let (tw, th) = text_size(scale, font, &text);
        bold_text(
            &mut img,
            ink_light,
            rect.left() + (rect.width() as i32 - tw as i32) / 2 - 1,
            rect.top() + (rect.height() as i32 - th as i32) / 2 - 4,
            scale,
            font,
            &text,
            2,
        );
    }
    img
}

fn mix_rgb(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let c = mix(a, b, t);
    [c[0], c[1], c[2]]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn goblin() -> ProxyFace {
        ProxyFace {
            name: "Goblin Shaman #2: \"Bob\"".into(),
            colors: vec![Color::Red, Color::Black],
            type_line: "Token Creature — Goblin Shaman".into(),
            keywords: vec!["Haste".into(), "Menace".into()],
            pt: Some((-1, 12)),
        }
    }

    /// The path round-trips, whatever the name holds, and never carries a
    /// character a file name or a Bevy asset path would choke on.
    #[test]
    fn a_proxy_path_round_trips() {
        let face = goblin();
        let path = proxy_asset_path(&face);
        let stem = path.strip_prefix("cards/").unwrap().strip_suffix(".png").unwrap();
        assert!(stem.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'), "{stem}");
        assert_eq!(parse_proxy_stem(stem), Some(face));
        assert_eq!(parse_proxy_stem("goblin"), None);
    }

    /// A face whose spec would make a file name too long drops keywords
    /// rather than fail to open.
    #[test]
    fn an_overlong_face_sheds_keywords() {
        let face = ProxyFace { keywords: vec!["Protection from everything".into(); 20], ..goblin() };
        let path = proxy_asset_path(&face);
        assert!(path.len() < 255, "{}", path.len());
        let parsed = parse_proxy_stem(path.strip_prefix("cards/").unwrap().strip_suffix(".png").unwrap()).unwrap();
        assert_eq!(parsed.name, face.name);
        assert!(parsed.keywords.len() < face.keywords.len());
    }

    #[test]
    fn a_face_draws_at_card_proportions() {
        let img = render_proxy(&goblin(), None);
        assert_eq!((img.width(), img.height()), (PROXY_W, PROXY_H));
    }
}
