//! Download card images from Scryfall and cache them in assets/cards/.

use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

/// Ensure card images exist locally for every (name, is_back_face) entry.
/// Downloads missing images from Scryfall's API. Blocks until done.
///
/// `is_back_face` flags MDFC back-face entries (Searstep Pathway,
/// Slitherbore Pathway). Scryfall's `format=image` returns the **front**
/// face by default even when queried by back-face name, so back-face
/// downloads explicitly request `&face=back`.
pub fn ensure_card_images(card_specs: &[(&str, bool)], assets_dir: &Path) {
    let cards_dir = assets_dir.join("cards");
    fs::create_dir_all(&cards_dir).expect("failed to create assets/cards/ directory");

    for (name, is_back) in card_specs {
        let filename = if *is_back {
            card_back_face_filename(name)
        } else {
            card_filename(name)
        };
        let path = cards_dir.join(&filename);
        if path.exists() {
            continue;
        }

        let face_label = if *is_back { " (back face)" } else { "" };
        println!("Downloading card image: {name}{face_label}...");
        match download_card_image(name, *is_back) {
            Ok(bytes) => {
                fs::write(&path, &bytes).expect("failed to write card image");
                println!("  Saved to {}", path.display());
            }
            Err(e) => {
                eprintln!("  Failed to download {name}: {e}");
            }
        }

        // Scryfall asks for ≤10 req/s; be polite
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

fn download_card_image(name: &str, is_back: bool) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let face_param = if is_back { "&face=back" } else { "" };
    let url = format!(
        "https://api.scryfall.com/cards/named?exact={}&format=image&version=png{face_param}",
        urlenccode(name)
    );

    let response = ureq::get(&url).call()?;
    let bytes = response.into_body().read_to_vec()?;
    Ok(bytes)
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
