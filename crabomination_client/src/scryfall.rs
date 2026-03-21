//! Download card images from Scryfall and cache them in assets/cards/.

use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

/// Ensure card images exist locally for every card name in the list.
/// Downloads missing images from Scryfall's API. Blocks until done.
pub fn ensure_card_images(card_names: &[&str], assets_dir: &Path) {
    let cards_dir = assets_dir.join("cards");
    fs::create_dir_all(&cards_dir).expect("failed to create assets/cards/ directory");

    for name in card_names {
        let filename = card_filename(name);
        let path = cards_dir.join(&filename);
        if path.exists() {
            continue;
        }

        println!("Downloading card image: {name}...");
        match download_card_image(name) {
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

/// Asset path relative to the assets/ root, for use with Bevy's AssetServer.
pub fn card_asset_path(name: &str) -> String {
    format!("cards/{}", card_filename(name))
}

fn download_card_image(name: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.scryfall.com/cards/named?exact={}&format=image&version=png",
        urlenccode(name)
    );

    let response = ureq::get(&url).call()?;
    let bytes = response.into_body().read_to_vec()?;
    Ok(bytes)
}

/// Minimal percent-encoding for card names in URLs.
fn urlenccode(s: &str) -> String {
    s.replace(' ', "+")
}
