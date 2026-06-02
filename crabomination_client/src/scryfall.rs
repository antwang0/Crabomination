//! Download card images from Scryfall and cache them in `assets/cards/`.
//!
//! # Design
//!
//! Each card we want to render is described by a [`CardImage`]. Single-
//! faced cards round-trip through Scryfall via `cards/named?exact=NAME`
//! plus `format=image`; multi-faced cards (transform / MDFC) need
//! special handling because Scryfall keys those by the **front** face
//! name and exposes the back face only through `face=back`. Naively
//! querying with the back-face name returns 404 for most MDFCs (the
//! back name isn't a registered card name). The robust path is:
//!
//! 1. Identify each card image as either a `Front(name)` or
//!    `MdfcBack { front, back }`.
//! 2. For `MdfcBack`, query by the **front** name with `face=back`,
//!    save under the **back** name's filename so the runtime can load
//!    it via [`card_back_face_asset_path`].
//! 3. On `exact` failure, fall back to `fuzzy` once before giving up
//!    â€” this catches cards whose Scryfall display name has
//!    apostrophe / quote / set-symbol punctuation we'd otherwise
//!    miss-encode.
//!
//! Callers build a `&[CardImage]` from the catalog walk in `main.rs`
//! (where the front-and-back relationship is naturally available) and
//! hand it to [`ensure_card_images`]. Fictional / catalog-invented
//! cards (no Scryfall printing) are listed in [`FICTIONAL_CARDS`] and
//! skipped silently.

use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use ab_glyph::{FontVec, PxScale};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_hollow_rect_mut, draw_text_mut, text_size};
use imageproc::rect::Rect;

/// One image to fetch. Built from the catalog walk; consumed by
/// [`ensure_card_images`].
#[derive(Debug, Clone)]
pub enum CardImage {
    /// Single-faced card. Looked up on Scryfall by `name`.
    Front(&'static str),
    /// Modal-double-faced / transform back face. Looked up by the
    /// **front** face's name with `face=back`, saved under the back
    /// face's filename so the asset loader can retrieve it via
    /// [`card_back_face_asset_path(back)`].
    MdfcBack {
        front: &'static str,
        back: &'static str,
    },
    /// Token (Clue / Treasure / Food / Blood / Bird / Citizen /
    /// Faerie / Giant). Fetched via Scryfall's search endpoint
    /// (`q=is:token+t:<name>`) since `cards/named?exact=Clue` 404s â€”
    /// "Clue" isn't a card name, it's a token type. Saved under
    /// `cards/<name>.png` so the runtime asset loader serves it
    /// when `Effect::CreateToken` adds a new token to the
    /// battlefield.
    Token { name: &'static str },
}

impl CardImage {
    /// Display name for log messages.
    fn label(&self) -> String {
        match self {
            CardImage::Front(n) => (*n).to_string(),
            CardImage::MdfcBack { front, back } => format!("{back} (back of {front})"),
            CardImage::Token { name } => format!("{name} token"),
        }
    }

    /// The card's own display name, used as the text on the white
    /// name-placeholder when there's no Scryfall art.
    fn display_name(&self) -> &str {
        match self {
            CardImage::Front(n)
            | CardImage::MdfcBack { back: n, .. }
            | CardImage::Token { name: n } => n,
        }
    }

    /// File name (relative to `assets/cards/`) where the downloaded
    /// image gets stored.
    fn filename(&self) -> String {
        match self {
            CardImage::Front(n) | CardImage::Token { name: n } => card_filename(n),
            CardImage::MdfcBack { back, .. } => card_back_face_filename(back),
        }
    }

    /// Whether the prefetcher should skip this image silently because
    /// it's not a real Scryfall printing. Tokens are *real* on
    /// Scryfall (just queried via `is:token`), so they aren't
    /// fictional â€” the cardback placeholder is reserved for
    /// genuinely-invented cards.
    fn is_fictional(&self) -> bool {
        match self {
            CardImage::Front(n) => is_fictional(n),
            CardImage::MdfcBack { front, back } => is_fictional(front) || is_fictional(back),
            CardImage::Token { .. } => false,
        }
    }
}

/// Card names that don't exist on Scryfall and should be skipped by
/// the prefetcher. Engine-invented MDFCs only â€” tokens are now
/// fetched via the real Scryfall token-search path (`is:token+t:â€¦`)
/// in [`download_token_image`].
const FICTIONAL_CARDS: &[&str] = &[
    "Sundering Eruption",
    "Mount Tyrhus",
];

fn is_fictional(name: &str) -> bool {
    FICTIONAL_CARDS.iter().any(|f| f.eq_ignore_ascii_case(name))
        // Catalog-synthesised STX cards have no Scryfall printing; the
        // exact set is determined offline by diffing the catalog's card
        // names against Scryfall's `catalog/card-names` (see the
        // `synthesized_cards` module). Stamp a placeholder rather than
        // firing thousands of doomed lookups on first prefetch.
        || crate::synthesized_cards::is_synthesized_card(name)
}

/// Ensure card images exist locally for every entry in `specs`.
/// Blocks until done. Idempotent: existing files are skipped, fresh
/// downloads are rate-limited to â‰¤10 req/s per Scryfall's guidance.
///
/// Cards with no Scryfall art — engine-invented synthesized cards, MDFC
/// backs that 404, etc. — get a generated **white card carrying the
/// card's name** as a placeholder file, so the runtime asset loader has
/// something to serve and doesn't spam `Path not found`. These are a few
/// KB each (vs. the 10 MB cardback copy this used to stamp, which bloated
/// `cards/` by tens of GB).
pub fn ensure_card_images(specs: &[CardImage], assets_dir: &Path) {
    let cards_dir = assets_dir.join("cards");
    fs::create_dir_all(&cards_dir).expect("failed to create assets/cards/ directory");
    // Load the UI font once for placeholder text (None → blank white card).
    let placeholder_font = load_placeholder_font(assets_dir);

    // Tallies for a one-line summary instead of per-card spam. The audit
    // catalog contains ~3500 synthesised STX cards that aren't on Scryfall;
    // logging each placeholder/404 individually floods the console.
    let mut downloaded = 0u32;
    let mut fictional = 0u32;
    let mut unavailable = 0u32;

    for spec in specs {
        let path = cards_dir.join(spec.filename());
        if path.exists() {
            continue;
        }
        if spec.is_fictional() {
            // Generate a white name-placeholder so the asset loader has
            // something to serve when this catalog-invented card enters
            // play (it has no Scryfall printing).
            if let Err(e) =
                write_name_placeholder(&path, spec.display_name(), placeholder_font.as_ref())
            {
                eprintln!("  Failed to write placeholder for {}: {e}", spec.label());
            } else {
                fictional += 1;
            }
            continue;
        }

        println!("Downloading card image: {}â€¦", spec.label());
        match download_card_image(spec) {
            Ok(bytes) => {
                fs::write(&path, &bytes).expect("failed to write card image");
                println!("  Saved to {}", path.display());
                downloaded += 1;
            }
            Err(e) => {
                // 404s are expected here: the audit catalog holds many
                // synthesised STX cards with clean, real-looking names
                // we can't pre-filter, so they only reveal themselves as
                // "not found" on the first prefetch. Write the white
                // name placeholder (so the runtime has a file to serve and so
                // re-runs skip via the `path.exists()` check above) and
                // count it â€” a per-card error line here is the flood.
                let is_404 = e
                    .downcast_ref::<LookupError>()
                    .is_some_and(|le| matches!(le, LookupError::NotFound));
                if !is_404 {
                    eprintln!("  Failed to download {}: {e}", spec.label());
                }
                unavailable += 1;
                if let Err(ph_err) =
                    write_name_placeholder(&path, spec.display_name(), placeholder_font.as_ref())
                {
                    eprintln!(
                        "  Also failed to write placeholder for {}: {ph_err}",
                        spec.label(),
                    );
                }
            }
        }

        thread::sleep(Duration::from_millis(120));
    }

    if downloaded + fictional + unavailable > 0 {
        println!(
            "Card image prefetch: {downloaded} downloaded, \
             {fictional} fictional placeholder(s), \
             {unavailable} unavailable (name placeholder)."
        );
    }
}

/// Generated name-placeholder dimensions, in Scryfall "normal" card
/// proportions (63 × 88).
const PLACEHOLDER_W: u32 = 488;
const PLACEHOLDER_H: u32 = 680;

/// Load the UI font once for placeholder text. Returns `None` if the font
/// file isn't where we expect — the placeholder then renders as a blank
/// white card (still far better than the old 10 MB cardback copy).
fn load_placeholder_font(assets_dir: &Path) -> Option<FontVec> {
    let bytes = fs::read(assets_dir.join(crate::theme::FONT_PATH)).ok()?;
    FontVec::try_from_vec(bytes).ok()
}

/// Write a white "card" PNG carrying `name` as centered, word-wrapped
/// text — the placeholder for cards with no Scryfall art (synthesized
/// cards, MDFC backs, 404s). A few KB each, vs. the 10 MB cardback copy
/// this replaced.
fn write_name_placeholder(
    path: &Path,
    name: &str,
    font: Option<&FontVec>,
) -> image::ImageResult<()> {
    let mut img =
        RgbaImage::from_pixel(PLACEHOLDER_W, PLACEHOLDER_H, Rgba([245, 245, 245, 255]));

    // Card frame: a couple of nested dark rectangles.
    let frame = Rgba([70, 70, 70, 255]);
    draw_hollow_rect_mut(
        &mut img,
        Rect::at(6, 6).of_size(PLACEHOLDER_W - 12, PLACEHOLDER_H - 12),
        frame,
    );
    draw_hollow_rect_mut(
        &mut img,
        Rect::at(7, 7).of_size(PLACEHOLDER_W - 14, PLACEHOLDER_H - 14),
        frame,
    );

    if let Some(font) = font {
        let ink = Rgba([25, 25, 25, 255]);
        let scale = PxScale::from(34.0);
        let margin = 30u32;
        let max_w = (PLACEHOLDER_W - margin * 2) as i32;

        // Greedy word-wrap measured against the real glyph metrics.
        let mut lines: Vec<String> = Vec::new();
        let mut cur = String::new();
        for word in name.split_whitespace() {
            let trial = if cur.is_empty() {
                word.to_string()
            } else {
                format!("{cur} {word}")
            };
            if text_size(scale, font, &trial).0 as i32 > max_w && !cur.is_empty() {
                lines.push(std::mem::take(&mut cur));
                cur = word.to_string();
            } else {
                cur = trial;
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
        }

        let line_h = 42i32;
        let total_h = line_h * lines.len() as i32;
        let mut y = (PLACEHOLDER_H as i32 - total_h) / 2;
        for line in &lines {
            let line_w = text_size(scale, font, line).0 as i32;
            let x = (PLACEHOLDER_W as i32 - line_w) / 2;
            draw_text_mut(&mut img, ink, x, y, scale, font, line);
            y += line_h;
        }
    }

    // Write atomically (temp file + rename) so a prefetch interrupted
    // mid-write never leaves a 0-byte placeholder that the `path.exists()`
    // skip would treat as done on the next run.
    let tmp = path.with_extension("png.tmp");
    img.save_with_format(&tmp, image::ImageFormat::Png)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Convert a card name to a filename: lowercase, spaces to underscores, .png extension.
/// Path separators (`/`, `\`) are also collapsed to underscores so split-card
/// names like "Wear // Tear" don't get interpreted as nested directories by
/// `fs::write` (which panics with NotFound when the implied parent dirs
/// don't exist).
pub fn card_filename(name: &str) -> String {
    format!("{}.png", sanitize_name(name))
}

/// Filename for an MDFC back-face image. The `_back` suffix avoids
/// colliding with a stale front-face download for the same name when
/// the prefetch is upgraded to pass `face=back` to Scryfall.
pub fn card_back_face_filename(name: &str) -> String {
    format!("{}_back.png", sanitize_name(name))
}

fn sanitize_name(name: &str) -> String {
    name.to_lowercase().replace([' ', '/', '\\'], "_")
}

/// Asset path relative to the assets/ root, for use with Bevy's AssetServer.
pub fn card_asset_path(name: &str) -> String {
    format!("cards/{}", card_filename(name))
}

/// Asset path for an MDFC back-face image.
pub fn card_back_face_asset_path(name: &str) -> String {
    format!("cards/{}", card_back_face_filename(name))
}

fn download_card_image(spec: &CardImage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match spec {
        CardImage::Token { name } => download_token_image(name),
        spec => {
            // Query parameters: which name to look up, and whether to ask
            // Scryfall for the back face.
            let (lookup_name, face_param) = match spec {
                CardImage::Front(n) => (*n, ""),
                CardImage::MdfcBack { front, .. } => (*front, "&face=back"),
                CardImage::Token { .. } => unreachable!("handled above"),
            };
            match try_lookup("exact", lookup_name, face_param) {
                Ok(bytes) => Ok(bytes),
                Err(LookupError::NotFound) => {
                    try_lookup("fuzzy", lookup_name, face_param).map_err(Into::into)
                }
                Err(e) => Err(e.into()),
            }
        }
    }
}

/// Tokens (Clue / Treasure / Bird / etc.) aren't card names on
/// Scryfall â€” they're identified by `is:token` plus a type filter.
/// Two-step fetch:
///
/// 1. `cards/search?q=is%3Atoken+t%3A<name>` returns a JSON list of
///    token printings; we pick the first result.
/// 2. Pull `image_uris.png` (or `image_uris.large` as fallback) and
///    download the actual image bytes.
///
/// Scryfall returns the token regardless of which set it came from,
/// so `unique=art` is plenty â€” we don't care about printing variants.
fn download_token_image(token_name: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let search_url = format!(
        "https://api.scryfall.com/cards/search?unique=art&q=is%3Atoken+t%3A{}",
        urlenccode(token_name),
    );
    let body = ureq::get(&search_url)
        .call()?
        .into_body()
        .read_to_string()?;
    let parsed: serde_json::Value = serde_json::from_str(&body)?;
    let first = parsed["data"]
        .as_array()
        .and_then(|a| a.first())
        .ok_or_else(|| format!("no token result for {token_name:?}"))?;
    let img_url = first["image_uris"]["png"]
        .as_str()
        .or_else(|| first["image_uris"]["large"].as_str())
        .or_else(|| first["image_uris"]["normal"].as_str())
        .ok_or_else(|| format!("token {token_name:?} has no image_uris"))?;
    let bytes = ureq::get(img_url).call()?.into_body().read_to_vec()?;
    Ok(bytes)
}

#[derive(Debug)]
enum LookupError {
    NotFound,
    Other(String),
}

impl std::fmt::Display for LookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LookupError::NotFound => write!(f, "not found on Scryfall"),
            LookupError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for LookupError {}

fn try_lookup(
    matcher: &'static str,
    lookup_name: &str,
    face_param: &str,
) -> Result<Vec<u8>, LookupError> {
    let url = format!(
        "https://api.scryfall.com/cards/named?{matcher}={}&format=image&version=png{face_param}",
        urlenccode(lookup_name),
    );
    let response = match ureq::get(&url).call() {
        Ok(r) => r,
        // ureq reports 404 as a body-bearing status error; bubble that
        // out as `NotFound` so the caller can decide whether to retry
        // with `fuzzy`. All other errors (network, 5xx, parse failure)
        // are terminal.
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("status: 404") || msg.contains("404 Not Found") {
                return Err(LookupError::NotFound);
            }
            return Err(LookupError::Other(msg));
        }
    };
    response
        .into_body()
        .read_to_vec()
        .map_err(|e| LookupError::Other(e.to_string()))
}

/// Percent-encode a card name for use in a Scryfall URL query parameter.
/// Spaces become `+`; all non-ASCII and reserved ASCII characters are encoded.
fn urlenccode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b' ' => out.push('+'),
            // Unreserved characters (RFC 3986) pass through unchanged.
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' => out.push(byte as char),
            // Everything else (including non-ASCII UTF-8 bytes) is encoded.
            b => { let _ = write!(out, "%{b:02X}"); }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_image_filename_matches_asset_path() {
        let spec = CardImage::Front("Lightning Bolt");
        assert_eq!(spec.filename(), "lightning_bolt.png");
        assert_eq!(card_asset_path("Lightning Bolt"), "cards/lightning_bolt.png");
    }

    #[test]
    fn synthesized_stx_cards_are_detected() {
        use crate::synthesized_cards::is_synthesized_card;
        // A spread of synthesised names: batch-suffixed, roman-numeral, a
        // clean-named one, and a real-looking mimic ("Augusta, Dean of
        // Order" — Scryfall only has "Augusta, Order Returned").
        for synth in [
            "Silverquill Stridemage (b125)",
            "Inkling Sentinel II",
            "Lorehold Vanguard (Batch 123)",
            "Witherbloom Soothsayer",
            "Bombastic Strixhaven Mage",
            "Augusta, Dean of Order",
        ] {
            assert!(is_synthesized_card(synth), "{synth} not detected");
        }
        // Case-insensitive.
        assert!(is_synthesized_card("witherbloom soothsayer"));
    }

    #[test]
    fn real_cards_are_not_flagged_synthesized() {
        use crate::synthesized_cards::is_synthesized_card;
        for real in [
            "Academic Dispute",
            "Anger",
            "Baleful Mastery",
            "Beaming Defiance",
            "Lightning Bolt",
            "Adrix and Nev, Twincasters",
            "Approach of the Second Sun",
            "Silverquill Command",
        ] {
            assert!(!is_synthesized_card(real), "{real} wrongly flagged");
        }
    }

    #[test]
    fn mdfc_back_filename_uses_back_name_with_suffix() {
        let spec = CardImage::MdfcBack {
            front: "Cragcrown Pathway",
            back: "Timbercrown Pathway",
        };
        assert_eq!(spec.filename(), "timbercrown_pathway_back.png");
    }

    #[test]
    fn split_card_name_collapses_path_separators() {
        // Split cards ("Wear // Tear", "Reduce // Rubble") embed `/` in
        // their printed names. Without sanitising, `fs::write` interprets
        // `cards/wear_//_tear.png` as a nested path and panics with
        // NotFound when the implied parent dirs don't exist.
        assert_eq!(card_filename("Wear // Tear"), "wear____tear.png");
        assert_eq!(card_asset_path("Reduce // Rubble"), "cards/reduce____rubble.png");
        assert_eq!(
            card_back_face_filename("Foo / Bar"),
            "foo___bar_back.png",
        );
    }

    #[test]
    fn fictional_cards_are_skipped_on_either_face() {
        let front = CardImage::Front("Sundering Eruption");
        assert!(front.is_fictional());
        let back = CardImage::MdfcBack {
            front: "Sundering Eruption",
            back: "Mount Tyrhus",
        };
        assert!(back.is_fictional());
    }

    #[test]
    fn tokens_are_not_fictional_and_use_token_filename() {
        // Tokens are real Scryfall printings (queried via
        // `is:token+t:<name>`); only invented cards trigger the
        // cardback placeholder.
        for name in ["Bird", "Citizen", "Clue", "Faerie", "Food", "Giant", "Blood", "Treasure"] {
            let spec = CardImage::Token { name };
            assert!(!spec.is_fictional(), "{name} token must not be fictional");
            assert_eq!(spec.filename(), format!("{}.png", name.to_lowercase()));
        }
    }

    #[test]
    fn token_label_is_disambiguated_in_logs() {
        let spec = CardImage::Token { name: "Clue" };
        assert_eq!(spec.label(), "Clue token");
    }

    #[test]
    fn ensure_card_images_writes_name_placeholder_for_fictional_cards() {
        use std::fs;
        // Use a temp asset dir so we don't pollute the repo.
        let tmp = std::env::temp_dir().join(format!(
            "crab-scryfall-test-{}",
            std::process::id(),
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("cards")).expect("temp setup");
        // No fonts dir here, so the placeholder renders as a blank white
        // card (the font-missing fallback) — still a valid PNG.

        let specs = vec![CardImage::Front("Mount Tyrhus")];
        ensure_card_images(&specs, &tmp);

        let path = tmp.join("cards").join("mount_tyrhus.png");
        assert!(
            path.exists(),
            "fictional card must get a name placeholder at {}",
            path.display(),
        );
        // It's a freshly generated PNG of placeholder dimensions — not a
        // 10 MB cardback copy.
        let img = image::open(&path).expect("placeholder must be a valid PNG");
        assert_eq!(
            (img.width(), img.height()),
            (PLACEHOLDER_W, PLACEHOLDER_H),
            "placeholder should be a card-proportioned canvas",
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn label_disambiguates_back_face_in_logs() {
        let spec = CardImage::MdfcBack {
            front: "Cragcrown Pathway",
            back: "Timbercrown Pathway",
        };
        assert_eq!(spec.label(), "Timbercrown Pathway (back of Cragcrown Pathway)");
    }
}
