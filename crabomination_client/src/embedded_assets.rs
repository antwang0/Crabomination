//! Core static assets compiled into the binary and materialized into the
//! configured asset directory at startup.
//!
//! `paths.asset_dir` is user-configurable (and defaults differ between
//! debug and release builds — see `config::PathsConfig`), so a fresh or
//! custom directory starts empty. Card art streams in from Scryfall, but
//! the UI font, the cardback and the symbol fonts have no download source
//! — without them a custom asset dir meant a broken client. They are small
//! (≈500 KB total), so embedding keeps any asset dir self-sufficient.

use std::fs;
use std::path::Path;

/// (relative asset path, bytes) for every embedded core asset.
const EMBEDDED: &[(&str, &[u8])] = &[
    ("cardback.png", include_bytes!("../assets/cardback.png")),
    (
        "fonts/MiranoExtendedFreebie-Light.ttf",
        include_bytes!("../assets/fonts/MiranoExtendedFreebie-Light.ttf"),
    ),
    // The symbol fallbacks (`theme::FALLBACK_FONT_PATHS`, ~36 KB together).
    (
        "fonts/fallback/NotoSansSymbols2.subset.ttf",
        include_bytes!("../assets/fonts/fallback/NotoSansSymbols2.subset.ttf"),
    ),
    (
        "fonts/fallback/NotoSansSymbols.subset.ttf",
        include_bytes!("../assets/fonts/fallback/NotoSansSymbols.subset.ttf"),
    ),
    (
        "fonts/fallback/NotoSansMath.subset.ttf",
        include_bytes!("../assets/fonts/fallback/NotoSansMath.subset.ttf"),
    ),
    (
        "fonts/fallback/NotoEmoji.subset.ttf",
        include_bytes!("../assets/fonts/fallback/NotoEmoji.subset.ttf"),
    ),
];

/// Write any missing core asset into `asset_dir`. Existing files are left
/// untouched (a user-swapped cardback or font survives upgrades). Failures
/// are logged, not fatal — the repo checkout already has the files.
pub fn materialize_core_assets(asset_dir: &Path) {
    for (rel, bytes) in EMBEDDED {
        let dest = asset_dir.join(rel);
        if dest.exists() {
            continue;
        }
        if let Some(parent) = dest.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            eprintln!("assets: create {} failed: {e}", parent.display());
            continue;
        }
        match fs::write(&dest, bytes) {
            Ok(()) => eprintln!("assets: materialized {}", dest.display()),
            Err(e) => eprintln!("assets: write {} failed: {e}", dest.display()),
        }
    }
}
