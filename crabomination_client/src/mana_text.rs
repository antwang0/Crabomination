//! Mana symbols in UI text: `{2}{W}{W}`, `{T}`, `{W/U}` drawn as pips.
//!
//! The engine writes every cost the Oracle way — `ManaCost::summary()`'s
//! `{3}{W}{W}`, an ability's `{2}{T}: Draw a card` — and the client printed
//! those strings as they came. [`ManaText`] draws one as a row: each
//! `{…}` a round pip in the printed symbol's colour, the rest as text. The
//! UI font has no mana glyphs, so a pip is a node: a circle with a letter or
//! number on it, a hybrid split corner to corner.
//!
//! Put [`ManaText`] on a node (or spawn [`mana_text`]) wherever a string may
//! carry a cost; [`sync_mana_text`] rebuilds the row when the string changes.

use bevy::prelude::*;
use crabomination::mana::Color as ManaColor;

use crate::theme::UiFonts;

/// One mana symbol, as far as drawing it goes.
#[derive(Clone, Debug, PartialEq)]
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

/// A run of a mana string: a word of text or a pip, and whether a space
/// follows it.
#[derive(Clone, Debug, PartialEq)]
pub enum Segment {
    Word(String, bool),
    Pip(Pip, bool),
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

/// Split one line of text into words and pips. A `{…}` that names no
/// symbol stays text, braces and all.
pub fn segments(line: &str) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut Vec<Segment>, space_after: bool| {
        if !word.is_empty() {
            out.push(Segment::Word(std::mem::take(word), space_after));
        }
    };
    let mut rest = line;
    while let Some(c) = rest.chars().next() {
        if c == '{'
            && let Some(end) = rest.find('}')
            && let Some(p) = pip(&rest[1..end])
        {
            flush(&mut word, &mut out, false);
            rest = &rest[end + 1..];
            out.push(Segment::Pip(p, rest.starts_with(' ')));
            continue;
        }
        if c == ' ' {
            flush(&mut word, &mut out, true);
        } else {
            word.push(c);
        }
        rest = &rest[c.len_utf8()..];
    }
    flush(&mut word, &mut out, false);
    out
}

/// Whether `text` has a mana symbol in it.
pub fn has_pips(text: &str) -> bool {
    text.lines().any(|line| segments(line).iter().any(|s| matches!(s, Segment::Pip(..))))
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

/// Mana text on a node: the node's children are the row [`sync_mana_text`]
/// builds from `text`. Newlines start new rows; `wrap` lets a row wrap
/// between words.
#[derive(Component, Clone)]
pub struct ManaText {
    pub text: String,
    pub size: f32,
    pub color: Color,
    pub wrap: bool,
    /// The text the children were last built from.
    built: Option<String>,
}

impl ManaText {
    pub fn new(text: impl Into<String>, size: f32, color: Color) -> Self {
        Self { text: text.into(), size, color, wrap: false, built: None }
    }

    pub fn wrapping(mut self) -> Self {
        self.wrap = true;
        self
    }

    /// The text the row draws.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Replace the text; the row is rebuilt only if it differs. Through a
    /// `Mut` the call itself marks the component changed, so a caller that
    /// runs every frame compares with [`Self::text`] first.
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

/// Bevy system: build each changed [`ManaText`]'s row.
pub fn sync_mana_text(
    mut commands: Commands,
    fonts: Res<UiFonts>,
    mut texts: Query<(Entity, &mut ManaText, &mut Node), Changed<ManaText>>,
) {
    for (entity, mut mana, mut node) in &mut texts {
        if mana.built.as_deref() == Some(mana.text.as_str()) {
            continue;
        }
        mana.built = Some(mana.text.clone());
        let lines: Vec<&str> = mana.text.split('\n').collect();
        node.flex_direction = if lines.len() > 1 { FlexDirection::Column } else { FlexDirection::Row };
        node.align_items = if lines.len() > 1 { AlignItems::Start } else { AlignItems::Center };
        if lines.len() == 1 {
            node.flex_wrap = if mana.wrap { FlexWrap::Wrap } else { FlexWrap::NoWrap };
        }
        commands.entity(entity).despawn_children();
        let style = (mana.size, mana.color, mana.wrap);
        commands.entity(entity).with_children(|parent| {
            if lines.len() == 1 {
                spawn_row_items(parent, &fonts, lines[0], style);
            } else {
                for line in lines {
                    let wrap = if style.2 { FlexWrap::Wrap } else { FlexWrap::NoWrap };
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: wrap,
                            align_items: AlignItems::Center,
                            // An empty line keeps its height.
                            min_height: Val::Px(style.0 * 1.2),
                            ..default()
                        })
                        .with_children(|row| spawn_row_items(row, &fonts, line, style));
                }
            }
        });
    }
}

/// The words and pips of one line, as siblings in a row.
fn spawn_row_items(parent: &mut ChildSpawnerCommands, fonts: &UiFonts, line: &str, (size, color, _): (f32, Color, bool)) {
    let space = Val::Px(size * 0.3);
    for segment in segments(line) {
        match segment {
            Segment::Word(word, space_after) => {
                parent.spawn((
                    Text::new(word),
                    fonts.tf(size),
                    TextColor(color),
                    TextLayout::no_wrap(),
                    Node { margin: UiRect::right(if space_after { space } else { Val::ZERO }), ..default() },
                    Pickable::IGNORE,
                ));
            }
            Segment::Pip(pip, space_after) => {
                spawn_pip(parent, fonts, &pip, size, if space_after { space } else { Val::Px(1.0) });
            }
        }
    }
}

/// One pip: a circle a little taller than the text's capitals, with its
/// glyph; a two-color one split corner to corner.
fn spawn_pip(parent: &mut ChildSpawnerCommands, fonts: &UiFonts, pip: &Pip, size: f32, after: Val) {
    let (a, b, glyph) = look(pip);
    let diameter = (size * 1.05).round();
    let mut circle = parent.spawn((
        Node {
            width: Val::Px(diameter),
            height: Val::Px(diameter),
            margin: UiRect { left: Val::Px(1.0), right: after, ..default() },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::MAX,
            flex_shrink: 0.0,
            ..default()
        },
        Pickable::IGNORE,
    ));
    if a == b {
        circle.insert(BackgroundColor(a));
    } else {
        circle.insert(BackgroundGradient(vec![Gradient::Linear(LinearGradient::to_bottom_right(vec![
            ColorStop::new(a, Val::Percent(50.0)),
            ColorStop::new(b, Val::Percent(50.0)),
        ]))]));
    }
    if !glyph.is_empty() {
        circle.with_children(|c| {
            c.spawn((
                Text::new(glyph),
                fonts.tf((size * 0.72).round().max(8.0)),
                TextColor(INK),
                TextLayout::no_wrap(),
                Pickable::IGNORE,
            ));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cost_splits_into_pips_and_words() {
        use Segment::{Pip as P, Word as W};
        assert_eq!(
            segments("{2}{W}{W}"),
            vec![
                P(Pip::Generic("2".into()), false),
                P(Pip::Colored(ManaColor::White), false),
                P(Pip::Colored(ManaColor::White), false),
            ]
        );
        assert_eq!(
            segments("{2}{T}: Draw a card"),
            vec![
                P(Pip::Generic("2".into()), false),
                P(Pip::Tap, false),
                W(":".into(), true),
                W("Draw".into(), true),
                W("a".into(), true),
                W("card".into(), false),
            ]
        );
        assert_eq!(
            segments("Pay {X} (+2 tax)"),
            vec![
                W("Pay".into(), true),
                P(Pip::Generic("X".into()), true),
                W("(+2".into(), true),
                W("tax)".into(), false),
            ]
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
        let pips = segments(&cost.summary()).iter().filter(|s| matches!(s, Segment::Pip(..))).count();
        assert_eq!(pips, cost.symbols.len(), "{}", cost.summary());
    }

    #[test]
    fn braces_that_name_no_symbol_stay_text() {
        assert_eq!(segments("{oops}"), vec![Segment::Word("{oops}".into(), false)]);
        assert!(!has_pips("Draw a card"));
        assert!(has_pips("Kicker {1}{R}"));
    }
}
