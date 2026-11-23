//! CR 903.4 — a card's *color identity*, the quantity Commander deck
//! construction is built on: the colours of mana symbols in its cost or
//! rules text plus CDA / colour indicator colours. 903.4c excludes reminder
//! text, 903.4d includes a DFC's back face, 903.4e an adventurer's
//! alternative characteristics.
//!
//! A land's *basic land types* are folded in too. The CR keeps them in a
//! separate restriction (903.5d), but its only consumer is the same subset
//! check, and folding matches Scryfall's `color_identity`, which the
//! whole-catalog audit ratchets against. See `basic_land_identity`.
//!
//! **Rules text, not just the cost** (duals, Moxen and Cabal Coffers carry
//! identity in abilities). The module walks the *serialized* definition, so
//! every mana symbol modelled anywhere — activation costs, produced mana,
//! keyword costs, nested faces — counts, as CR 903.4 says. Same device as
//! `audit::dead_capabilities`: a hand-written match over ~90 fields drifts.
//!
//! Reminder text rarely becomes a `ManaSymbol` at all; the exception is an
//! inline-expanded keyword whose cost is printed only in reminder text (see
//! `is_reminder_only_cost`).

use serde_json::Value;

use crate::card::CardDefinition;
use crate::mana::{Color, ColorSet};

/// CR 903.4 — the card's color identity over both faces.
pub fn color_identity(def: &CardDefinition) -> ColorSet {
    let mut out = ColorSet::empty();
    basic_land_identity(def, &mut out);
    // `to_value` fails only on non-string map keys or non-finite floats,
    // neither in a `CardDefinition`; but this runs on a Commander mana-tap
    // path, so fall back to the basic-land half rather than abort a game
    // (`debug_assert` keeps it loud in tests).
    let v = serde_json::to_value(def).unwrap_or(Value::Null);
    debug_assert!(!v.is_null(), "CardDefinition serializes: {}", def.name);
    walk(&v, &mut out);
    out
}

/// CR 903.5d — "a card with a basic land type may be included only if each
/// color of mana it could produce is in the commander's color identity",
/// folded into identity (see the module header): Gingerbread Cabin is green
/// though its mana ability comes from its type.
///
/// Typed, not part of the serde walk: only the card's *own* land types
/// count, while "search for a Forest card" or a Forest token also serialize
/// a `land_types` list.
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
            // CR 903.4c / 702.99 — Extort's `{W/B}` is reminder text (Crypt
            // Ghast is mono-black) though the engine expands it into a real
            // hybrid `MayPay`. "This ability doesn't affect its color
            // identity" (Fallaji Wayfarer's CDA) says so itself.
            if m.get("description")
                .and_then(Value::as_str)
                .is_some_and(|d| d.contains("doesn't affect its color identity"))
            {
                return;
            }
            if m.get("description").and_then(Value::as_str).is_some_and(is_reminder_only_cost) {
                return;
            }
            // CR 111.10 / 903.4 — a predefined token's abilities are reminder
            // text on the card that makes it: "create a Heartwood token (… with
            // '{T}: Add {R} or {G}.')" leaves Hungering Puppetbeast mono-green.
            if m.get("name").and_then(Value::as_str).is_some_and(is_reminder_text_token) && m.contains_key("card_types") {
                return;
            }
            // CR 305.6 — a basic land type's intrinsic mana ability, printed by
            // the engine as an activated ability: on the card it's
            // `basic_land_identity`'s; on a token the card makes (Staff of
            // Titania) it's no symbol on the card at all.
            let intrinsic = basic_colors_of(m);
            // CR 111.3 / 903.4 — a token's defined mana cost is reminder text
            // ("It's a {3}{W}{W} … Baneslayer Angel" leaves Balefang mono-red).
            // `colors` + `card_types` is a `TokenDefinition`'s shape.
            let token = m.contains_key("colors") && m.contains_key("card_types");
            for (tag, p) in m {
                if token && tag == "cost" {
                    continue;
                }
                if tag == "activated_abilities" && !intrinsic.is_empty() {
                    for ab in p.as_array().into_iter().flatten() {
                        if !is_intrinsic_land_mana(ab, &intrinsic) {
                            walk(ab, out);
                        }
                    }
                    continue;
                }
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
                    // ColorlessHybrid(color) — {C/W} is white (CR 903.4).
                    "ColorlessHybrid" => color_of(p, out),
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
                    // A printed "add {R} … until end of combat" (Fire Lord
                    // Ozai). The firebending keyword's own `Firebend` is built
                    // in combat, never serialized: its {R} is reminder text.
                    "Firebend" => out.insert(Color::Red),
                    // ExtraManaKind::Fixed — "adds an additional {R}" (Gauntlet
                    // of Might, Wild Growth) is a printed symbol.
                    "Fixed" => color_of(p, out),
                    // "For each {G}{G} spent to cast it" (Clay Champion) — the
                    // spent-mana count names a printed symbol.
                    "SourceCastManaOfColor" => color_of(p, out),
                    // CR 903.4 — the color indicator, and a characteristic-
                    // defining color (the Kobolds' red, Transguild Courier).
                    "color_indicator" | "color_override" => each_color(p, out),
                    // CR 903.4 / 604.3 — Transguild Courier: a self-targeting
                    // all-colours CDA puts all five in its identity; making
                    // *other* permanents all colours doesn't.
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
        // "You may pay {W}{U}{B}{R}{G} rather than pay the mana cost" (Jodah,
        // Fist of Suns): a unit variant, so its five symbols are the name.
        Value::String(s) if s == "FiveColorAlternativeCost" => {
            for c in Color::ALL {
                out.insert(c);
            }
        }
        _ => {}
    }
}

/// The basic land types' colors of a serialized land definition (card or
/// token); empty for anything else.
fn basic_colors_of(m: &serde_json::Map<String, Value>) -> Vec<&'static str> {
    let is_land = m.get("card_types").and_then(Value::as_array).is_some_and(|t| t.iter().any(|x| x == "Land"));
    if !is_land {
        return Vec::new();
    }
    let types = m.get("subtypes").and_then(|s| s.get("land_types")).and_then(Value::as_array);
    types
        .into_iter()
        .flatten()
        .filter_map(|t| match t.as_str()? {
            "Plains" => Some("White"),
            "Island" => Some("Blue"),
            "Swamp" => Some("Black"),
            "Mountain" => Some("Red"),
            "Forest" => Some("Green"),
            _ => None,
        })
        .collect()
}

/// `{T}: Add` one of `colors` and nothing else — a basic type's own ability.
fn is_intrinsic_land_mana(ab: &Value, colors: &[&str]) -> bool {
    let Some(pool) = ab.get("effect").and_then(|e| e.get("AddMana")).and_then(|a| a.get("pool")) else {
        return false;
    };
    ab.get("tap_cost").and_then(Value::as_bool) == Some(true)
        && pool
            .get("Colors")
            .and_then(Value::as_array)
            .is_some_and(|cs| cs.iter().all(|c| c.as_str().is_some_and(|c| colors.contains(&c))))
}

/// CR 903.4c — costs the engine models structurally but the card only prints
/// in reminder text. Keyed on the shortcut's own description so the list is
/// exactly the shortcuts that mint them; extend it if another keyword joins.
fn is_reminder_only_cost(description: &str) -> bool {
    description.starts_with("Extort")
}

/// CR 111.10 — the predefined tokens whose abilities print a colored mana
/// symbol (only in reminder text). Treasure, Gold and the rest add mana of any
/// color or colorless, which no walk tag reads.
fn is_reminder_text_token(name: &str) -> bool {
    name == "Heartwood"
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

    /// CR 903.5d — a land with a basic land type is bounded by the colors of
    /// mana it can produce, so a dual land's identity is both its colors even
    /// though its mana cost is empty. The pre-2026-09 walk read `def.cost`
    /// only and called Tundra colorless.
    #[test]
    fn cr_903_5d_dual_land_identity_comes_from_its_mana_abilities() {
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

    /// CR 903.4 — "You may pay {W}{U}{B}{R}{G} rather than pay the mana
    /// cost" is a rules-text cost: Jodah is five colors, not his {1}{U}{R}{W}.
    #[test]
    fn cr_903_4_a_five_color_alternative_cost_is_five_colors() {
        assert_eq!(color_identity(&catalog::jodah_archmage_eternal()).len(), 5);
    }

    /// CR 111.10 / 903.4 — a Heartwood token's "{T}: Add {R} or {G}" is
    /// reminder text on the card that makes it: Hungering Puppetbeast is
    /// mono-green.
    #[test]
    fn cr_903_4_a_predefined_tokens_reminder_mana_adds_no_identity() {
        let id = color_identity(&catalog::hungering_puppetbeast());
        assert!(id.contains(Color::Green));
        assert_eq!(id.len(), 1, "the Heartwood's red is reminder text");
    }

    /// The tag table is safe only while each name belongs to one enum
    /// (`{"Protection":"White"}` must not read as identity). Regenerate by
    /// grepping the variant names if it fails.
    #[test]
    fn tag_names_are_unambiguous() {
        // Protection from a color is not color identity (CR 903.4 counts mana
        // symbols and indicators, not color words).
        let id = color_identity(&catalog::white_knight());
        assert!(!id.contains(Color::Black), "protection from black is not identity");
        assert!(id.contains(Color::White));
    }
}
