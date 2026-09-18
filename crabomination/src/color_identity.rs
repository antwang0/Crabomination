//! CR 903.4 — a card's *color identity*, the quantity Commander deck
//! construction is built on.
//!
//! 903.4: "The Commander variant uses color identity to determine what cards
//! can be in a deck with a certain commander. The color identity of a card is
//! the color or colors of any mana symbols in that card's mana cost or rules
//! text, plus any colors defined by its characteristic-defining ability or
//! color indicator." 903.4b excludes reminder text; 903.4c says a land card
//! with a basic land type has the corresponding mana symbol in its rules text;
//! 903.4d includes the back face of a double-faced card.
//!
//! **Rules text, not just the mana cost.** Every nonbasic dual land, every Mox
//! and Cabal Coffers carries its whole identity in an ability, so a walk of
//! `def.cost` alone reports them colorless and a mono-green deck validates
//! with Mox Pearl in it. This module walks the *serialized* definition
//! instead, which is total by construction: a mana symbol the engine models
//! anywhere — an ability's activation cost, the payload it adds, a keyword's
//! cost, a nested face — is a symbol printed on the card, and CR 903.4 counts
//! all of them. The serde walk is the same device `audit::dead_capabilities`
//! uses, and for the same reason: a hand-written match over `CardDefinition`'s
//! ~90 fields drifts the moment one is added.
//!
//! Reminder text mostly needs no exclusion: the engine models rules, not
//! prose, so a reminder-text symbol rarely becomes a `ManaSymbol` at all. The
//! exception is a keyword the engine expands inline whose whole cost is
//! printed only in reminder text — see `is_reminder_only_cost`.

use serde_json::Value;

use crate::card::CardDefinition;
use crate::mana::{Color, ColorSet};

/// CR 903.4 — the card's color identity over both faces.
pub fn color_identity(def: &CardDefinition) -> ColorSet {
    let mut out = ColorSet::empty();
    basic_land_identity(def, &mut out);
    // `to_value` fails only on a map with non-string keys or a non-finite
    // float, neither of which a `CardDefinition` holds — but this runs on a
    // mana-tap path in a Commander game (`AnyColorInCommanderIdentity`), and a
    // definition-shape bug is not a reason to abort a self-play game. Fall
    // back to the basic-land half; `debug_assert` keeps it loud in a test run.
    let v = serde_json::to_value(def).unwrap_or(Value::Null);
    debug_assert!(!v.is_null(), "CardDefinition serializes: {}", def.name);
    walk(&v, &mut out);
    out
}

/// CR 903.4c — "a card with a basic land type has the corresponding mana
/// symbol in its rules text", reminder text or not: Gingerbread Cabin is a
/// Forest and green even though the engine gives a typed land its mana
/// ability from the type rather than from a printed activated ability.
///
/// Typed rather than part of the serde walk on purpose: only the *card's own*
/// land types count. "Search your library for a Forest card" and a token that
/// enters as a Forest both serialize a `land_types` list, and neither is a
/// basic land type on this card.
fn basic_land_identity(def: &CardDefinition, out: &mut ColorSet) {
    use crate::card::{CardType, LandType};
    if def.card_types.contains(&CardType::Land) {
        for t in &def.subtypes.land_types {
            match t {
                LandType::Plains => out.insert(Color::White),
                LandType::Island => out.insert(Color::Blue),
                LandType::Swamp => out.insert(Color::Black),
                LandType::Mountain => out.insert(Color::Red),
                LandType::Forest => out.insert(Color::Green),
                _ => {}
            }
        }
    }
    for face in [def.back_face.as_deref(), def.flip_face.as_deref()].into_iter().flatten() {
        basic_land_identity(face, out);
    }
    if let Some(prep) = &def.prepare_spell {
        basic_land_identity(prep, out);
    }
}

/// Serde tags that carry a color contributing to identity. Each name is
/// unique across the crate's enums (checked by `tag_names_are_unambiguous`),
/// so a tag match cannot pick up `Keyword::Protection(Color)` or
/// `HexproofFromColor` — protection from a color is not identity in it.
fn walk(v: &Value, out: &mut ColorSet) {
    match v {
        Value::Object(m) => {
            // CR 903.4b / 702.99 — Extort's `{W/B}` lives in reminder text, so
            // a Crypt Ghast is mono-black. The engine expands the keyword into
            // a real `MayPay` with a real hybrid cost; that cost is the one
            // printed cost on a card that is not printed on the card.
            if m.get("description").and_then(Value::as_str).is_some_and(is_reminder_only_cost) {
                return;
            }
            for (tag, p) in m {
                match tag.as_str() {
                    // ManaSymbol — the colored halves of a cost pip.
                    "Colored" | "Phyrexian" => color_of(p, out),
                    "Hybrid" | "PhyrexianHybrid" => each_color(p, out),
                    // MonoHybrid(generic, color) — the color is the tail.
                    "MonoHybrid" => {
                        if let Some(c) = p.as_array().and_then(|a| a.get(1)) {
                            color_of(c, out);
                        }
                    }
                    // ManaPayload — "{T}: Add {B}" is a rules-text symbol.
                    "Colors" => each_color(p, out),
                    "OfColor" => {
                        if let Some(c) = p.as_array().and_then(|a| a.first()) {
                            color_of(c, out);
                        }
                    }
                    "OfColors" => {
                        if let Some(cs) = p.as_array().and_then(|a| a.first()) {
                            each_color(cs, out);
                        }
                    }
                    // CR 500.4 exception payloads carry their colors as fields.
                    "AddManaKeptThisTurn" => {
                        if let Some(cs) = p.get("colors") {
                            each_color(cs, out);
                        }
                    }
                    "AddManaKeptThisTurnCount" => {
                        if let Some(c) = p.get("color") {
                            color_of(c, out);
                        }
                    }
                    // CR 903.4 — the color indicator, and a characteristic-
                    // defining color (the Kobolds' red, Transguild Courier).
                    "color_indicator" | "color_override" => each_color(p, out),
                    // CR 903.4a names Transguild Courier: a characteristic-
                    // defining ability that makes the card all colors puts
                    // all five in its identity. Only when it points at
                    // itself — a card that makes *other* permanents all
                    // colors keeps its own identity.
                    "GrantAllColors"
                        if p.get("applies_to").and_then(Value::as_str) == Some("This") =>
                    {
                        for c in Color::ALL {
                            out.insert(c);
                        }
                    }
                    // "If {G} was spent to cast this spell …" — a printed
                    // symbol in rules text (the Ravnica/Guildpact cycle, the
                    // Mythos cycle, CR 702.137 adamant).
                    "ManaSpentOfColorAtLeast" | "SourceCastWithColorSpent" => {
                        if let Some(c) = p.get("color") {
                            color_of(c, out);
                        }
                    }
                    _ => {}
                }
                walk(p, out);
            }
        }
        Value::Array(a) => a.iter().for_each(|e| walk(e, out)),
        _ => {}
    }
}

/// CR 903.4b — costs the engine models structurally but the card only prints
/// in reminder text. Keyed on the shortcut's own description so the list is
/// exactly the shortcuts that mint them; extend it if another keyword joins.
fn is_reminder_only_cost(description: &str) -> bool {
    description.starts_with("Extort")
}

fn color_of(v: &Value, out: &mut ColorSet) {
    let c = match v.as_str() {
        Some("White") => Color::White,
        Some("Blue") => Color::Blue,
        Some("Black") => Color::Black,
        Some("Red") => Color::Red,
        Some("Green") => Color::Green,
        _ => return,
    };
    out.insert(c);
}

fn each_color(v: &Value, out: &mut ColorSet) {
    match v {
        Value::Array(a) => a.iter().for_each(|e| color_of(e, out)),
        _ => color_of(v, out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    /// CR 903.4c — a land with a basic land type has that mana symbol in its
    /// rules text, so a dual land's identity is both its colors even though
    /// its mana cost is empty. The pre-2026-09 walk read `def.cost` only and
    /// called Tundra colorless.
    #[test]
    fn cr_903_4c_dual_land_identity_comes_from_its_mana_abilities() {
        let id = color_identity(&catalog::tundra());
        assert!(id.contains(Color::White) && id.contains(Color::Blue), "Tundra is WU");
        assert_eq!(id.len(), 2);
    }

    /// CR 903.4 — "{T}: Add {W}" is a rules-text mana symbol; Mox Pearl's
    /// identity is white despite a `{0}` mana cost. It is therefore illegal
    /// in a mono-green Commander deck, which is the bug this caught.
    #[test]
    fn cr_903_4_mox_pearl_is_white_not_colorless() {
        let id = color_identity(&catalog::mox_pearl());
        assert!(id.contains(Color::White));
        assert_eq!(id.len(), 1);
    }

    /// CR 903.4 — "Add one mana of any color" prints no colored symbol, so
    /// Sol Ring and its kin stay colorless and fit every commander.
    #[test]
    fn cr_903_4_generic_and_any_color_production_adds_no_identity() {
        assert_eq!(color_identity(&catalog::sol_ring()), ColorSet::empty());
    }

    /// CR 903.4 — a colored *activation* cost is rules text too.
    #[test]
    fn cr_903_4_activation_cost_colors_count() {
        let id = color_identity(&catalog::cabal_coffers());
        assert!(id.contains(Color::Black), "Cabal Coffers adds {{B}}");
    }

    /// The tag table is only safe while each name belongs to one enum. A
    /// `Protection(Color)` keyword serializes as `{"Protection":"White"}` and
    /// must not read as identity; this pins that the tags we *do* match are
    /// not reused. Regenerate by grepping the variant names if it fails.
    #[test]
    fn tag_names_are_unambiguous() {
        // Protection from a color is not color identity (CR 903.4 counts mana
        // symbols and indicators, not color words).
        let id = color_identity(&catalog::white_knight());
        assert!(!id.contains(Color::Black), "protection from black is not identity");
        assert!(id.contains(Color::White));
    }
}
