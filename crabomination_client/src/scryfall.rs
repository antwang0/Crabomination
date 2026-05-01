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
//!    — this catches cards whose Scryfall display name has
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
    /// (`q=is:token+t:<name>`) since `cards/named?exact=Clue` 404s —
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
    /// fictional — the cardback placeholder is reserved for
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
/// the prefetcher. Engine-invented MDFCs only — tokens are now
/// fetched via the real Scryfall token-search path (`is:token+t:…`)
/// in [`download_token_image`].
const FICTIONAL_CARDS: &[&str] = &[
    "Sundering Eruption",
    "Mount Tyrhus",
];

fn is_fictional(name: &str) -> bool {
    FICTIONAL_CARDS.iter().any(|f| f.eq_ignore_ascii_case(name))
}

/// Ensure card images exist locally for every entry in `specs`.
/// Blocks until done. Idempotent: existing files are skipped, fresh
/// downloads are rate-limited to ≤10 req/s per Scryfall's guidance.
///
/// Fictional cards (engine-invented MDFCs + token names) get a
/// **cardback copy** as a placeholder file so the runtime asset
/// loader doesn't spam `Path not found` errors when those cards
/// enter play. The placeholder is visually a card-back; replacing it
/// with a real token image is a future improvement.
pub fn ensure_card_images(specs: &[CardImage], assets_dir: &Path) {
    let cards_dir = assets_dir.join("cards");
    fs::create_dir_all(&cards_dir).expect("failed to create assets/cards/ directory");
    let cardback_placeholder = cards_dir.join("cardback.png");

    for spec in specs {
        let path = cards_dir.join(spec.filename());
        if path.exists() {
            continue;
        }
        if spec.is_fictional() {
            // Stamp a cardback copy so the asset loader has *something*
            // to serve when the token / fictional card enters play.
            // Best-effort: if the cardback itself isn't on disk yet
            // we silently skip — Bevy will still log a missing-asset
            // warning for that single token instance, but the rest of
            // the prefetch keeps running.
            if cardback_placeholder.exists() {
                if let Err(e) = fs::copy(&cardback_placeholder, &path) {
                    eprintln!(
                        "  Failed to write placeholder for {}: {e}",
                        spec.label(),
                    );
                } else {
                    eprintln!(
                        "  Placeholder ({} → cardback): not a real Scryfall card",
                        spec.label(),
                    );
                }
            } else {
                eprintln!(
                    "  Skipping {}: not a real Scryfall card (cardback placeholder missing too)",
                    spec.label(),
                );
            }
            continue;
        }

        println!("Downloading card image: {}…", spec.label());
        match download_card_image(spec) {
            Ok(bytes) => {
                fs::write(&path, &bytes).expect("failed to write card image");
                println!("  Saved to {}", path.display());
            }
            Err(e) => {
                eprintln!("  Failed to download {}: {e}", spec.label());
                // Last-resort placeholder: stamp a copy of the
                // cardback so the runtime asset loader doesn't spam
                // `Path not found` for this card. Without this,
                // failed token fetches (`treasure.png`, etc.) render
                // as a missing-asset error every frame the token is
                // on the battlefield.
                if cardback_placeholder.exists() {
                    if let Err(copy_err) = fs::copy(&cardback_placeholder, &path) {
                        eprintln!(
                            "  Also failed to stamp cardback placeholder for {}: {copy_err}",
                            spec.label(),
                        );
                    } else {
                        eprintln!(
                            "  Stamped cardback placeholder for {} so the runtime has a file to serve",
                            spec.label(),
                        );
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(120));
    }
}

/// Convert a card name to a filename: lowercase, spaces to underscores, .png extension.
pub fn card_filename(name: &str) -> String {
    format!("{}.png", name.to_lowercase().replace(' ', "_"))
}

/// Filename for an MDFC back-face image. The `_back` suffix avoids
/// colliding with a stale front-face download for the same name when
/// the prefetch is upgraded to pass `face=back` to Scryfall.
pub fn card_back_face_filename(name: &str) -> String {
    format!("{}_back.png", name.to_lowercase().replace(' ', "_"))
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
                Err(e) if matches!(e, LookupError::NotFound) => {
                    try_lookup("fuzzy", lookup_name, face_param).map_err(Into::into)
                }
                Err(e) => Err(e.into()),
            }
        }
    }
}

/// Tokens (Clue / Treasure / Bird / etc.) aren't card names on
/// Scryfall — they're identified by `is:token` plus a type filter.
/// Two-step fetch:
///
/// 1. `cards/search?q=is%3Atoken+t%3A<name>` returns a JSON list of
///    token printings; we pick the first result.
/// 2. Pull `image_uris.png` (or `image_uris.large` as fallback) and
///    download the actual image bytes.
///
/// Scryfall returns the token regardless of which set it came from,
/// so `unique=art` is plenty — we don't care about printing variants.
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
    fn mdfc_back_filename_uses_back_name_with_suffix() {
        let spec = CardImage::MdfcBack {
            front: "Cragcrown Pathway",
            back: "Timbercrown Pathway",
        };
        assert_eq!(spec.filename(), "timbercrown_pathway_back.png");
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
    fn ensure_card_images_stamps_cardback_for_fictional_cards() {
        use std::fs;
        // Use a temp asset dir so we don't pollute the repo.
        let tmp = std::env::temp_dir().join(format!(
            "crab-scryfall-test-{}",
            std::process::id(),
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("cards")).expect("temp setup");
        // Stamp a 1-byte fake cardback so the placeholder logic has
        // something to copy from.
        fs::write(tmp.join("cards").join("cardback.png"), b"FAKE").expect("write fake cardback");

        let specs = vec![CardImage::Front("Mount Tyrhus")];
        ensure_card_images(&specs, &tmp);

        let path = tmp.join("cards").join("mount_tyrhus.png");
        assert!(
            path.exists(),
            "fictional card must get a cardback placeholder at {}",
            path.display(),
        );
        assert_eq!(
            fs::read(&path).unwrap(),
            b"FAKE",
            "placeholder should be a copy of the cardback bytes",
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
