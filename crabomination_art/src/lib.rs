//! Card art: the images a catalog needs, their names on disk, and — native
//! only — downloading them from Scryfall into an asset directory's `cards/`.
//!
//! The client runs this as its launch prefetch (`crabomination_client`'s
//! `scryfall` module keeps the Bevy half: the placeholder reader and the hot
//! reload); `fill_art` runs it to fill the asset dir a browser bundle is
//! served from, since the browser cannot download or cache art itself.
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
//!    it via [`card_back_face_filename`].
//! 3. On `exact` failure, fall back to `fuzzy` once before giving up
//!    -- this catches cards whose Scryfall display name has
//!    apostrophe / quote / set-symbol punctuation we'd otherwise
//!    miss-encode.
//!
//! Callers walk their catalog into a [`SpecSet`] (where the front-and-back
//! relationship is naturally available) and hand its specs to
//! [`ensure_card_images`]. Fictional / catalog-invented cards (no Scryfall
//! printing) are listed in [`FICTIONAL_CARDS`] and `synthesized_cards`, and
//! skipped silently.

// On wasm the whole download half is compiled out (no ureq/threads/fs); the
// types it shares with the native path would otherwise warn as dead.
#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

#[cfg(not(target_arch = "wasm32"))]
use std::fmt::Write;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use crabomination_base::card::CardDefinition;

#[cfg(not(target_arch = "wasm32"))]
mod embedded_assets;
mod synthesized_cards;

#[cfg(not(target_arch = "wasm32"))]
pub use embedded_assets::materialize_core_assets;

/// One image to fetch. Built from the catalog walk; consumed by
/// [`ensure_card_images`].
#[derive(Debug, Clone)]
pub enum CardImage {
    /// Single-faced card. Looked up on Scryfall by `name`.
    Front(&'static str),
    /// Modal-double-faced / transform back face. Looked up by the
    /// **front** face's name with `face=back`, saved under the back
    /// face's filename ([`card_back_face_filename`]) so the asset loader
    /// can retrieve it.
    MdfcBack {
        front: &'static str,
        back: &'static str,
    },
    /// Token (Clue / Treasure / Food / Blood / Bird / Citizen /
    /// Faerie / Giant). Fetched via Scryfall's search endpoint
    /// (`q=is:token+t:<name>`) since `cards/named?exact=Clue` 404s --
    /// "Clue" isn't a card name, it's a token type. Saved under
    /// `cards/<name>.jpg` so the runtime asset loader serves it
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
    /// fictional -- the cardback placeholder is reserved for
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
/// the prefetcher. Engine-invented MDFCs only -- tokens are now
/// fetched via the real Scryfall token-search path (`is:token+t:...`)
/// in [`download_token_image`].
const FICTIONAL_CARDS: &[&str] = &["Sundering Eruption", "Mount Tyrhus"];

fn is_fictional(name: &str) -> bool {
    FICTIONAL_CARDS.iter().any(|f| f.eq_ignore_ascii_case(name))
        // Catalog-synthesised STX cards have no Scryfall printing; the
        // exact set is determined offline by diffing the catalog's card
        // names against Scryfall's `catalog/card-names` (see the
        // `synthesized_cards` module). Stamp a placeholder rather than
        // firing thousands of doomed lookups on first prefetch.
        || crate::synthesized_cards::is_synthesized_card(name)
}

/// Manifest (under `assets/cards/`) listing the stored filenames of cards
/// Scryfall has no fetchable art for, so the prefetcher skips them on
/// subsequent runs. Delete it to force a re-check (e.g. after a card is
/// added to Scryfall).
const UNAVAILABLE_MANIFEST: &str = ".unavailable.txt";

/// Load the negative-cache manifest into a set of filenames. Missing /
/// unreadable file → empty set (every card gets re-checked).
#[cfg(not(target_arch = "wasm32"))]
fn load_unavailable(cards_dir: &Path) -> std::collections::HashSet<String> {
    fs::read_to_string(cards_dir.join(UNAVAILABLE_MANIFEST))
        .map(|s| {
            s.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Append `filename` to the negative-cache manifest. Best-effort: a write
/// failure just means the card is re-checked next launch.
#[cfg(not(target_arch = "wasm32"))]
fn mark_unavailable(cards_dir: &Path, filename: &str) {
    use std::io::Write;
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(cards_dir.join(UNAVAILABLE_MANIFEST))
    {
        let _ = writeln!(f, "{filename}");
    }
}

/// Token names: created mid-game by `Effect::CreateToken` factories,
/// never present in deck definitions, so a catalog walk doesn't see them.
/// They use a separate Scryfall query path (`is:token !"<name>"`) since the
/// bare type name doesn't resolve on `cards/named`.
pub const TOKEN_NAMES: &[&str] = &[
    // Cube / demo tokens.
    "Bird", "Citizen", "Faerie", "Giant",
    "Clue", "Treasure", "Food", "Blood",
    // Cube-extras created by Soldier / Beast / Construct / Elephant /
    // Cat token cards (Raise the Alarm, Beast Within, Karn Scion of
    // Urza, Mascot Exhibition).
    "Soldier", "Beast", "Construct", "Elephant", "Cat",
    // Strixhaven / SoS-specific tokens. Without these the client
    // prefetch never downloads art for Fractal Anomaly's payload, the
    // Inkling tokens that Eager Glyphmage and friends produce, the
    // Pest tokens (Pest Mascot, Pest Summoning), or the R/W Spirit
    // tokens (Antiquities on the Loose, Group Project), so they
    // render with a missing-asset placeholder in the 3-D view.
    "Fractal", "Inkling", "Pest", "Spirit",
    // Prismari Elemental (Artistic Process mode 2, Visionary's Dance,
    // Muse's Encouragement, the rest of the elemental_token() callers).
    // Was missing from the prefetch list, so the freshly-minted token
    // entered with a placeholder front face on the battlefield.
    "Elemental",
];

/// The images a set of card definitions needs, deduplicated: every front,
/// every MDFC back with its front→back link (so the prefetcher can query
/// Scryfall by the front name with `face=back` — querying by back name alone
/// 404s for most MDFCs), and every SOS preparation card's inset spell.
///
/// Callers visit the demo (Modern) decks and every card the registry knows
/// about — cube, SoS, the *full* STX catalog, plus the xtra / Theros sets.
/// Cube matches roll a random deck per seat after startup, so the prefetch
/// can't be narrowed to "cards in this match"; the full union keeps Scryfall
/// off the critical path of gameplay. The registry is a superset of every
/// deck *and* the audit catalog (and the Lessons sideboard), so audit-mode
/// cards and Learn-fetched Lessons (Academic Dispute, Anger, …) get art too;
/// an earlier walk of only cube + SoS left the many STX cards outside those
/// pools without an image.
///
/// The walk is the caller's, not a function here: naming the catalog in this
/// library would link all of it into the library's test binary (253 MB
/// against 22 MB).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct SpecSet {
    fronts: std::collections::HashSet<&'static str>,
    mdfc_pairs: std::collections::HashSet<(&'static str, &'static str)>,
}

#[cfg(not(target_arch = "wasm32"))]
impl SpecSet {
    pub fn visit(&mut self, def: &CardDefinition) {
        self.fronts.insert(def.name);
        if let Some(back) = def.back_face.as_ref() {
            self.mdfc_pairs.insert((def.name, back.name));
        }
        // SOS preparation cards: the cast copy appears on the stack under
        // the inset spell's own name, so prefetch it as a plain front —
        // Scryfall's `cards/named` resolves face names (returning the
        // full preparation card's art, which is the right image).
        if let Some(prep) = def.prepare_spell.as_ref() {
            self.fronts.insert(prep.name);
        }
    }

    /// The specs, with [`TOKEN_NAMES`] appended.
    pub fn into_specs(self) -> Vec<CardImage> {
        let mut specs: Vec<CardImage> = self.fronts.into_iter().map(CardImage::Front).collect();
        specs.extend(self.mdfc_pairs.into_iter().map(|(front, back)| CardImage::MdfcBack { front, back }));
        specs.extend(TOKEN_NAMES.iter().map(|name| CardImage::Token { name }));
        specs
    }
}

/// Shared state between a prefetch running on another thread and whoever
/// watches it: counters for a progress readout, a "finished" latch, and the
/// asset-relative paths of images written since last drained (the client
/// hands those to `AssetServer::reload` so placeholders swap to real art as
/// downloads land).
#[derive(Clone, Default)]
pub struct Progress {
    pub done: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    pub total: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    pub finished: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub completed: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    /// The counters are [`convert_png_art`]'s, not the downloads'.
    pub converting: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

/// Blocking variant without progress reporting (tests).
#[cfg_attr(not(test), allow(dead_code))]
#[cfg(not(target_arch = "wasm32"))]
pub fn ensure_card_images(specs: &[CardImage], assets_dir: &Path) {
    ensure_card_images_with_progress(specs, assets_dir, &Progress::default());
}

/// Image filenames the running app has actually asked to render, recorded
/// through [`note_image_requested`] (the client's `card_asset_path` /
/// `card_back_face_asset_path` call it) and drained by
/// the prefetch loop. The catalog is thousands of cards deep at Scryfall's
/// rate limit, so a card on the table would otherwise wait behind an hour
/// of art nobody is looking at.
static REQUESTED_IMAGES: std::sync::Mutex<Option<std::collections::HashSet<String>>> =
    std::sync::Mutex::new(None);

/// Note that `filename` is wanted on screen now, so the prefetch queue can
/// jump it ahead of the rest of the catalog.
pub fn note_image_requested(filename: String) {
    if let Ok(mut set) = REQUESTED_IMAGES.lock() {
        set.get_or_insert_with(Default::default).insert(filename);
    }
}

/// Index into `pending` of the first spec the app has asked to render, if
/// any. Consumed entries are dropped so a name is only jumped once.
#[cfg(not(target_arch = "wasm32"))]
fn next_requested(pending: &[&CardImage]) -> Option<usize> {
    let mut guard = REQUESTED_IMAGES.lock().ok()?;
    let wanted = guard.as_mut()?;
    if wanted.is_empty() {
        return None;
    }
    let i = pending.iter().position(|s| wanted.contains(&s.filename()))?;
    wanted.remove(&pending[i].filename());
    Some(i)
}

/// `ensure_card_images` with live progress reporting — run on a background
/// thread so launch never blocks on the network (the placeholder reader
/// serves name placeholders until the real art lands and is reloaded).
#[cfg(not(target_arch = "wasm32"))]
pub fn ensure_card_images_with_progress(
    specs: &[CardImage],
    assets_dir: &Path,
    progress: &Progress,
) {
    use std::sync::atomic::Ordering;
    let cards_dir = assets_dir.join("cards");
    // This runs on a detached thread: a panic here would silently kill the
    // prefetch and leave the menu progress bar incomplete forever. Treat an
    // unwritable assets dir as "prefetch disabled" instead.
    if let Err(e) = fs::create_dir_all(&cards_dir) {
        eprintln!(
            "card prefetch disabled: cannot create {}: {e}",
            cards_dir.display()
        );
        progress
            .finished
            .store(true, std::sync::atomic::Ordering::Relaxed);
        return;
    }

    convert_png_art(&cards_dir, progress);

    // Negative cache: card names Scryfall has already told us it can't serve
    // (404 / 422). Loaded from `cards/.unavailable.txt` so a doomed card
    // isn't re-requested on every launch — that repeated, pointless traffic
    // is what trips Scryfall's rate limiter (the 429 cascades). Keyed by the
    // stored filename, which is unique per fetch.
    let unavailable_set = load_unavailable(&cards_dir);

    // Tallies for a one-line summary instead of per-card spam. The audit
    // catalog contains ~3500 synthesised STX cards that aren't on Scryfall;
    // logging each placeholder/404 individually floods the console.
    let mut fictional = 0u32;
    let mut unavailable = 0u32;

    // Work queue: the genuinely fetchable, genuinely missing set, and the
    // progress denominator. The skipped classes are tallied here rather than
    // inside the download loop so that loop is free to reorder.
    let mut pending: Vec<&CardImage> = Vec::new();
    for spec in specs {
        if cards_dir.join(spec.filename()).exists() {
            continue;
        }
        if spec.is_fictional() {
            // No file written: catalog-invented cards have no Scryfall art,
            // and the client's `CardPlaceholderReader` synthesizes a name-placeholder
            // on demand at load time (no 10 MB-of-copies on disk).
            fictional += 1;
            continue;
        }
        if unavailable_set.contains(&spec.filename()) {
            // Known-missing from a previous run — skip the doomed request.
            unavailable += 1;
            continue;
        }
        pending.push(spec);
    }
    progress.total.store(pending.len(), Ordering::Relaxed);

    // A large fill resolves image URLs from the bulk file; without it every
    // card takes the per-card lookup, which is slow but within the limits.
    let bulk = (pending.len() >= BULK_THRESHOLD)
        .then(|| {
            println!("Resolving {} card images from Scryfall's bulk card file...", pending.len());
            BulkIndex::download()
                .inspect_err(|e| eprintln!("  bulk card file unavailable ({e}); looking cards up one at a time"))
                .ok()
        })
        .flatten();
    let queue = std::sync::Mutex::new((0usize, pending));
    let manifest = std::sync::Mutex::new(());
    let fetched = std::sync::atomic::AtomicU32::new(0);
    let failed = std::sync::atomic::AtomicU32::new(0);
    thread::scope(|scope| {
        for _ in 0..DOWNLOAD_WORKERS {
            scope.spawn(|| loop {
                let spec = {
                    let Ok(mut queue) = queue.lock() else { return };
                    let (next, pending) = &mut *queue;
                    if *next >= pending.len() {
                        return;
                    }
                    // Anything the app has put on screen jumps the queue; a
                    // swap keeps the rest in catalog order and never drops a
                    // spec.
                    if let Some(i) = next_requested(&pending[*next..]) {
                        pending.swap(*next, *next + i);
                    }
                    *next += 1;
                    pending[*next - 1]
                };
                let path = cards_dir.join(spec.filename());
                match download_card_image(spec, bulk.as_ref()) {
                    Ok(bytes) => match write_whole(&path, &bytes) {
                        Ok(()) => {
                            println!("Saved {} to {}", spec.label(), path.display());
                            fetched.fetch_add(1, Ordering::Relaxed);
                            if let Ok(mut completed) = progress.completed.lock() {
                                completed.push(format!("cards/{}", spec.filename()));
                            }
                        }
                        // Full disk, permissions, invalid filename — log and
                        // keep going; the placeholder reader covers the
                        // missing art.
                        Err(e) => {
                            eprintln!("  failed to write {}: {e}", path.display());
                            failed.fetch_add(1, Ordering::Relaxed);
                        }
                    },
                    // Definitively no art on Scryfall (404 or 422). Negative-
                    // cache it so future launches skip it; the client's
                    // `CardPlaceholderReader` serves a name placeholder for
                    // the missing path. Suppressed from the log — these are
                    // expected for the synthesised audit cards.
                    Err(LookupError::NotFound) | Err(LookupError::Unavailable) => {
                        let _held = manifest.lock();
                        mark_unavailable(&cards_dir, &spec.filename());
                        failed.fetch_add(1, Ordering::Relaxed);
                    }
                    // Transient (network / persistent rate-limit). Not cached,
                    // so it's retried next launch; surface it since it may be
                    // actionable.
                    Err(LookupError::Other(msg)) => {
                        eprintln!("  Failed to download {}: {msg}", spec.label());
                        failed.fetch_add(1, Ordering::Relaxed);
                    }
                }
                progress.done.fetch_add(1, Ordering::Relaxed);
            });
        }
    });
    let downloaded = fetched.into_inner();
    unavailable += failed.into_inner();
    progress.finished.store(true, Ordering::Relaxed);

    if downloaded + fictional + unavailable > 0 {
        println!(
            "Card image prefetch: {downloaded} downloaded, \
             {fictional} fictional, {unavailable} unavailable \
             (both served runtime placeholders)."
        );
    }
}

/// Write `bytes` to `path` through a sibling temporary file, so an exit
/// mid-write leaves no truncated image for the next launch to take as cached.
#[cfg(not(target_arch = "wasm32"))]
fn write_whole(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let part = path.with_extension("part");
    fs::write(&part, bytes)?;
    fs::rename(&part, path)
}

/// Width of Scryfall's `large` image, which the art cache is kept at.
#[cfg(not(target_arch = "wasm32"))]
const ART_WIDTH: u32 = 672;

/// `bytes` as a JPEG: passed through when they already are one, re-encoded
/// otherwise (a token printing with only a PNG, art cached before the
/// switch) — scaled down to [`ART_WIDTH`] and at quality 85, which is about
/// what Scryfall serves: 15 % of the PNG's size over a 40-card sample, where
/// a full-size quality 90 was 26 % and read no better. Transparency — a
/// PNG's rounded corners — is laid over black, as in Scryfall's own JPEGs;
/// the card mesh is rounded anyway.
#[cfg(not(target_arch = "wasm32"))]
fn as_jpeg(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok(bytes);
    }
    let rgba = image::load_from_memory(&bytes).map_err(|e| e.to_string())?.to_rgba8();
    let mut rgb = image::RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
        let [r, g, b, a] = rgba.get_pixel(x, y).0;
        let over_black = |c: u8| ((c as u16 * a as u16 + 127) / 255) as u8;
        image::Rgb([over_black(r), over_black(g), over_black(b)])
    });
    let (w, h) = rgb.dimensions();
    if w > ART_WIDTH {
        let scaled_h = (h * ART_WIDTH + w / 2) / w;
        rgb = image::imageops::resize(&rgb, ART_WIDTH, scaled_h, image::imageops::FilterType::CatmullRom);
    }
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 85)
        .encode_image(&rgb)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

/// Convert art cached as PNG — every download before 2026-09-30, ~1.3 MB a
/// card, 28 GB for the catalog — to the JPEG [`card_filename`] now names,
/// on half the machine's threads, and point the negative cache at the new
/// names. Each converted file is announced through `progress.completed`, so a
/// placeholder already on screen swaps to it. A PNG that will not decode is
/// dropped, and the prefetch downloads the card again; one whose JPEG cannot
/// be written (a full disk) is kept for the next launch. Returns the count
/// converted.
#[cfg(not(target_arch = "wasm32"))]
pub fn convert_png_art(cards_dir: &Path, progress: &Progress) -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let Ok(entries) = fs::read_dir(cards_dir) else { return 0 };
    let legacy: Vec<std::path::PathBuf> = entries
        .filter_map(|e| Some(e.ok()?.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "png"))
        .collect();
    if legacy.is_empty() {
        return 0;
    }
    let manifest = cards_dir.join(UNAVAILABLE_MANIFEST);
    if let Ok(listed) = fs::read_to_string(&manifest) {
        let renamed: String = listed
            .lines()
            .map(|l| l.strip_suffix(".png").map_or_else(|| format!("{l}\n"), |stem| format!("{stem}.jpg\n")))
            .collect();
        let _ = fs::write(&manifest, renamed);
    }

    progress.converting.store(true, Ordering::Relaxed);
    progress.done.store(0, Ordering::Relaxed);
    progress.total.store(legacy.len(), Ordering::Relaxed);
    let next = AtomicUsize::new(0);
    let converted = AtomicUsize::new(0);
    let threads = thread::available_parallelism().map_or(1, |n| n.get() / 2).max(1);
    thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                while let Some(png) = legacy.get(next.fetch_add(1, Ordering::Relaxed)) {
                    progress.done.fetch_add(1, Ordering::Relaxed);
                    let jpg = png.with_extension("jpg");
                    if !jpg.exists() {
                        match fs::read(png).map_err(|e| e.to_string()).and_then(as_jpeg) {
                            Ok(bytes) => {
                                if let Err(e) = write_whole(&jpg, &bytes) {
                                    // Keep the PNG; the next launch tries again.
                                    eprintln!("  failed to write {}: {e}", jpg.display());
                                    continue;
                                }
                            }
                            Err(e) => {
                                eprintln!("  dropping unreadable cached art {}: {e}", png.display());
                                let _ = fs::remove_file(png);
                                continue;
                            }
                        }
                    }
                    let _ = fs::remove_file(png);
                    converted.fetch_add(1, Ordering::Relaxed);
                    if let (Ok(mut completed), Some(name)) = (progress.completed.lock(), jpg.file_name()) {
                        completed.push(format!("cards/{}", name.to_string_lossy()));
                    }
                }
            });
        }
    });
    progress.converting.store(false, Ordering::Relaxed);
    progress.done.store(0, Ordering::Relaxed);
    progress.total.store(0, Ordering::Relaxed);
    let converted = converted.into_inner();
    println!("Card art cache: converted {converted} of {} PNGs to JPEG.", legacy.len());
    converted
}

/// Convert a card name to a filename: lowercase, spaces to underscores, .jpg extension.
/// Path separators (`/`, `\`) are also collapsed to underscores so split-card
/// names like "Wear // Tear" don't get interpreted as nested directories by
/// `fs::write` (which panics with NotFound when the implied parent dirs
/// don't exist).
///
/// Art is Scryfall's `large` JPEG (672 × 936, ~120 KB), not its PNG
/// (745 × 1040, ~1.2 MB): the largest a card is ever drawn is the 230 px
/// hover preview, and the PNGs of a full catalog were 28 GB on disk. Art
/// cached as PNG before 2026-09-30 is converted by [`convert_png_art`].
pub fn card_filename(name: &str) -> String {
    format!("{}.jpg", sanitize_name(name))
}

/// Filename for an MDFC back-face image. The `_back` suffix avoids
/// colliding with a stale front-face download for the same name when
/// the prefetch is upgraded to pass `face=back` to Scryfall.
pub fn card_back_face_filename(name: &str) -> String {
    format!("{}_back.jpg", sanitize_name(name))
}

fn sanitize_name(name: &str) -> String {
    // Beyond path separators, strip every Windows-invalid filename character
    // (Circle of Protection: Red, Henzie "Toolbox" Torre, Who/What/When/...)
    // — an invalid name makes fs::write fail on Windows. All replaced chars
    // become '_', which the client's `card_name_from_asset_path` treats as a word
    // separator, so the lossy reverse mapping stays consistent.
    name.to_lowercase()
        .replace([' ', '/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
}

/// The name to query Scryfall with, stripped of a trailing deck-variant
/// disambiguator the catalog appends to keep names unique (e.g. "Putrefy
/// (Modern)" → "Putrefy"). The full catalog name is still used for the local
/// filename, so the saved image lines up with the runtime asset path.
///
/// Only known format-name suffixes are stripped — the synthesized-audit
/// `(bNNN)` suffix is deliberately left alone, since those cards aren't on
/// Scryfall and stripping could make one accidentally resolve to a real
/// card's art.
#[cfg(not(target_arch = "wasm32"))]
fn scryfall_lookup_name(name: &str) -> &str {
    const FORMAT_SUFFIXES: &[&str] = &[
        " (Modern)",
        " (Legacy)",
        " (Vintage)",
        " (Standard)",
        " (Pioneer)",
        " (Pauper)",
        " (Commander)",
        " (Historic)",
    ];
    for suffix in FORMAT_SUFFIXES {
        if let Some(base) = name.strip_suffix(suffix) {
            return base;
        }
    }
    name
}

/// Missing images at or above which the prefetch resolves image URLs from
/// Scryfall's bulk card file instead of one `/cards/named` lookup each.
/// Scryfall: "If you need to … resolve a large number of card images, you
/// must use the bulk data files" — and at `/cards/named`'s 2 a second, a
/// full catalog was a 3½-hour fill. Below this, 25 MB costs more than the
/// lookups do.
#[cfg(not(target_arch = "wasm32"))]
const BULK_THRESHOLD: usize = 50;

/// Name → image URLs, from Scryfall's Oracle Cards bulk file: one object per
/// Oracle ID, ~39 k lines of JSONL, 25 MB gzipped on the unlimited
/// `data.scryfall.io`, streamed rather than held. It resolved all but 27 of
/// the catalog's 22,161 real names (2026-10-07); those — names Scryfall
/// prints with diacritics the catalog drops, invented cards missing from
/// `synthesized_cards` — take the per-card lookup and its fuzzy match.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct BulkIndex {
    cards: Vec<BulkImages>,
    /// Lowercased full names ("Wear // Tear").
    by_name: std::collections::HashMap<String, usize>,
    /// Lowercased face names, consulted after `by_name`.
    by_face: std::collections::HashMap<String, usize>,
}

/// A card's `large` image URLs: its one image, or one a face.
#[cfg(not(target_arch = "wasm32"))]
struct BulkImages {
    whole: Option<String>,
    faces: Vec<Option<String>>,
}

/// The fields of a bulk card object the index reads.
#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Deserialize)]
struct BulkCard {
    name: String,
    layout: String,
    image_uris: Option<BulkUris>,
    #[serde(default)]
    card_faces: Vec<BulkFace>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Deserialize)]
struct BulkFace {
    name: String,
    image_uris: Option<BulkUris>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Deserialize)]
struct BulkUris {
    large: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
impl BulkIndex {
    /// Fetch and index the current Oracle Cards file.
    fn download() -> Result<Self, String> {
        use std::io::BufRead;
        #[derive(serde::Deserialize)]
        struct Entry {
            jsonl_download_uri: String,
        }
        let meta = scryfall_get_bytes("https://api.scryfall.com/bulk-data/oracle-cards").map_err(|e| e.to_string())?;
        let entry: Entry = serde_json::from_slice(&meta).map_err(|e| e.to_string())?;
        let body = AGENT
            .get(&entry.jsonl_download_uri)
            .header("User-Agent", SCRYFALL_USER_AGENT)
            .call()
            .map_err(|e| e.to_string())?
            .into_body()
            .into_reader();
        let mut index = Self::default();
        for line in std::io::BufReader::new(flate2::read::GzDecoder::new(body)).lines() {
            index.add_line(&line.map_err(|e| e.to_string())?)?;
        }
        Ok(index)
    }

    /// Index one line of the JSONL (blank lines and array brackets skipped,
    /// so the plain-JSON form of the file reads too).
    fn add_line(&mut self, line: &str) -> Result<(), String> {
        let line = line.trim().trim_end_matches(',');
        if line.is_empty() || line == "[" || line == "]" {
            return Ok(());
        }
        let card: BulkCard = serde_json::from_str(line).map_err(|e| e.to_string())?;
        // Art-series cards and tokens reuse real cards' names (an art card
        // "Brightglass Gearhulk // Brightglass Gearhulk", a "Soldier" token);
        // indexed, one would stand in for the card. Tokens are fetched by
        // search instead (`download_token_image`).
        if matches!(card.layout.as_str(), "art_series" | "token" | "double_faced_token" | "emblem") {
            return Ok(());
        }
        let i = self.cards.len();
        self.by_name.entry(card.name.to_lowercase()).or_insert(i);
        for face in &card.card_faces {
            self.by_face.entry(face.name.to_lowercase()).or_insert(i);
        }
        let large = |uris: Option<BulkUris>| uris.and_then(|u| u.large);
        self.cards.push(BulkImages {
            whole: large(card.image_uris),
            faces: card.card_faces.into_iter().map(|f| large(f.image_uris)).collect(),
        });
        Ok(())
    }

    /// The image `spec` names: what `cards/named?format=image` would have
    /// redirected to. `None` sends the spec to that lookup.
    fn image_url(&self, spec: &CardImage) -> Option<&str> {
        let card = |name: &str| {
            let key = scryfall_lookup_name(name).to_lowercase();
            self.by_name.get(&key).or_else(|| self.by_face.get(&key)).map(|&i| &self.cards[i])
        };
        match spec {
            // A card's one image, else its front face's — also when it is
            // named by another face, as `cards/named` does.
            CardImage::Front(name) => {
                let c = card(name)?;
                c.whole.as_deref().or_else(|| c.faces.first().and_then(|f| f.as_deref()))
            }
            CardImage::MdfcBack { front, .. } => card(front)?.faces.get(1)?.as_deref(),
            CardImage::Token { .. } => None,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn download_card_image(spec: &CardImage, bulk: Option<&BulkIndex>) -> Result<Vec<u8>, LookupError> {
    if let Some(url) = bulk.and_then(|b| b.image_url(spec)) {
        return as_jpeg(scryfall_get_bytes(url)?).map_err(LookupError::Other);
    }
    match spec {
        CardImage::Token { name } => download_token_image(name),
        spec => {
            // Query parameters: which name to look up, and whether to ask
            // Scryfall for the back face.
            let (lookup_name, face_param) = match spec {
                CardImage::Front(n) => (scryfall_lookup_name(n), ""),
                CardImage::MdfcBack { front, .. } => (scryfall_lookup_name(front), "&face=back"),
                CardImage::Token { .. } => unreachable!("handled above"),
            };
            let bytes = match try_lookup("exact", lookup_name, face_param) {
                // Only an exact 404 is worth a fuzzy retry. A 422
                // (`Unavailable`, e.g. no back face) won't be fixed by
                // fuzzing the name, so don't spend a second request on it.
                Err(LookupError::NotFound) => try_lookup("fuzzy", lookup_name, face_param),
                other => other,
            }?;
            as_jpeg(bytes).map_err(LookupError::Other)
        }
    }
}

/// Tokens (Clue / Treasure / Bird / etc.) aren't card names on
/// Scryfall -- they're identified by `is:token` plus the token's name.
/// Two-step fetch:
///
/// 1. `cards/search?q=is:token !"<name>"` returns the token printings
///    *named exactly* `<name>`; we pick the first result. It searched by
///    type (`t:<name>`) until 2026-09-29, and the first token with the
///    type in its line is often a *named* one: "Soldier" fetched Ajani's
///    Pridemate (a 2/2 Cat Soldier), which then stood in for every 1/1
///    Soldier token. See [`purge_type_matched_token_art`].
/// 2. Pull `image_uris.large` (or `normal`, then `png`, as fallbacks) and
///    download the actual image bytes.
///
/// Scryfall returns the token regardless of which set it came from,
/// so `unique=art` is plenty -- we don't care about printing variants.
#[cfg(not(target_arch = "wasm32"))]
fn download_token_image(token_name: &str) -> Result<Vec<u8>, LookupError> {
    let search_url = format!(
        "https://api.scryfall.com/cards/search?unique=art&q=is%3Atoken+{}",
        urlenccode(&format!("!\"{token_name}\"")),
    );
    let body = scryfall_get_bytes(&search_url)?;
    let parsed: serde_json::Value =
        serde_json::from_slice(&body).map_err(|e| LookupError::Other(e.to_string()))?;
    // The first single-faced printing named exactly this. `!"name"` also
    // matches a double-faced token with a face of that name ("Goblin //
    // Soldier", "Dinosaur // Treasure"), and those sort first and carry
    // their art per face. None — treat as unavailable (placeholder), not a
    // transient error, so it's negative-cached.
    let first = parsed["data"]
        .as_array()
        .and_then(|a| {
            a.iter().find(|c| c["name"].as_str() == Some(token_name) && c["image_uris"].is_object())
        })
        .ok_or(LookupError::Unavailable)?;
    let img_url = ["large", "normal", "png"]
        .iter()
        .find_map(|v| first["image_uris"][v].as_str())
        .ok_or(LookupError::Unavailable)?;
    as_jpeg(scryfall_get_bytes(img_url)?).map_err(LookupError::Other)
}

/// Delete token art fetched by the old type-matched search (see
/// [`download_token_image`]) once, so the prefetch fetches it again by exact
/// name. A marker file records that it ran. Art the user placed by hand
/// under the same names goes too — it is re-fetched, not lost for good.
#[cfg(not(target_arch = "wasm32"))]
pub fn purge_type_matched_token_art(asset_dir: &Path, token_names: &[&str]) {
    let cards = asset_dir.join("cards");
    let marker = cards.join(".token_art_by_exact_name");
    if marker.exists() || !cards.is_dir() {
        return;
    }
    // Each name as it is stored now and as it was before the JPEG switch.
    let files: std::collections::HashSet<String> = token_names
        .iter()
        .flat_map(|n| {
            let file = card_filename(n);
            let legacy = Path::new(&file).with_extension("png").to_string_lossy().into_owned();
            [file, legacy]
        })
        .collect();
    for file in &files {
        let _ = fs::remove_file(cards.join(file));
    }
    // A token the type search found nothing for may exist by name.
    let manifest = cards.join(UNAVAILABLE_MANIFEST);
    if let Ok(listed) = fs::read_to_string(&manifest) {
        let kept: String = listed.lines().filter(|l| !files.contains(l.trim())).map(|l| format!("{l}\n")).collect();
        let _ = fs::write(&manifest, kept);
    }
    let _ = fs::write(&marker, b"token art is fetched by exact name (is:token !\"name\")\n");
}

#[derive(Debug)]
enum LookupError {
    /// HTTP 404 — the card name isn't on Scryfall. The caller may retry with
    /// a fuzzy match before giving up.
    NotFound,
    /// HTTP 4xx other than 404 (chiefly 422 — e.g. asking for the back face
    /// of a single-faced or invented card). The request can't be served;
    /// don't fuzzy-retry. Like `NotFound`, this is a *definitive* "no art",
    /// so the prefetcher negative-caches it.
    Unavailable,
    /// Transient failure (network error, or a 429 / 5xx that persisted past
    /// the retry budget). Not negative-cached — retried on the next launch.
    Other(String),
}

impl std::fmt::Display for LookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LookupError::NotFound => write!(f, "not found on Scryfall"),
            LookupError::Unavailable => write!(f, "no fetchable image on Scryfall"),
            LookupError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for LookupError {}

#[cfg(not(target_arch = "wasm32"))]
fn try_lookup(
    matcher: &'static str,
    lookup_name: &str,
    face_param: &str,
) -> Result<Vec<u8>, LookupError> {
    let url = format!(
        "https://api.scryfall.com/cards/named?{matcher}={}&format=image&version=large{face_param}",
        urlenccode(lookup_name),
    );
    scryfall_get_bytes(&url)
}

/// Identify the client to Scryfall. Their API guidelines ask every caller to
/// send a descriptive `User-Agent` (anonymous traffic is rate-limited more
/// aggressively, which is the usual source of the 429 cascades).
const SCRYFALL_USER_AGENT: &str = concat!(
    "Crabomination/",
    env!("CARGO_PKG_VERSION"),
    " (card-image prefetch)"
);

/// How many times to retry a single request that comes back rate-limited
/// (HTTP 429) or with a transient 5xx before giving up on that card. The
/// negative cache stops doomed cards being re-requested at all on later
/// launches, which is what actually relieves the rate limit.
const MAX_RATE_LIMIT_RETRIES: u32 = 8;

/// GET `url` from Scryfall and return the response body bytes, sending the
/// required identifying headers and retrying with exponential backoff on a
/// 429 (rate limit) or 5xx (transient server error). A 404 maps to
/// `NotFound` so the caller can fall back to a fuzzy lookup; other errors are
/// terminal. Without the retry a single 429 used to fail every remaining card
/// in the batch — Scryfall rate-limits the *whole* run once it trips.
#[cfg(not(target_arch = "wasm32"))]
fn scryfall_get_bytes(url: &str) -> Result<Vec<u8>, LookupError> {
    let mut attempt = 0u32;
    loop {
        if url.starts_with("https://api.scryfall.com/") {
            pace_api_request();
        }
        match AGENT.get(url)
            .header("User-Agent", SCRYFALL_USER_AGENT)
            .header("Accept", "*/*")
            .call()
        {
            Ok(response) => {
                return response
                    .into_body()
                    .read_to_vec()
                    .map_err(|e| LookupError::Other(e.to_string()));
            }
            Err(ureq::Error::StatusCode(404)) => return Err(LookupError::NotFound),
            // 429 / 5xx — transient. Back off and retry within the budget;
            // give up as `Other` (not negative-cached) once exhausted.
            Err(ureq::Error::StatusCode(code)) if code == 429 || (500..=599).contains(&code) => {
                if attempt >= MAX_RATE_LIMIT_RETRIES {
                    return Err(LookupError::Other(format!(
                        "http status: {code} (persisted past {MAX_RATE_LIMIT_RETRIES} retries)"
                    )));
                }
                attempt += 1;
                // 30 s, 60 s, 120 s, ... capped at 32 min. A 429 locks the
                // caller out for 30 s, and Scryfall bans clients that keep
                // requesting through it — the old 8 s first retry did.
                let wait = Duration::from_millis(30_000u64 << (attempt - 1).min(6));
                eprintln!(
                    "  HTTP {code} (rate limited); retry {attempt}/{MAX_RATE_LIMIT_RETRIES} after {}ms",
                    wait.as_millis()
                );
                thread::sleep(wait);
            }
            // Any other 4xx (notably 422 — no back face, malformed request):
            // the request can't be served, so it's definitively unavailable.
            Err(ureq::Error::StatusCode(code)) if (400..=499).contains(&code) => {
                return Err(LookupError::Unavailable);
            }
            Err(e) => return Err(LookupError::Other(e.to_string())),
        }
    }
}

/// One agent for every request, so its pool keeps the connections to
/// `api.scryfall.com` and the `cards.scryfall.io` image host it redirects to
/// open. `ureq::get` runs each request on a use-once agent — a fresh TCP + TLS
/// handshake to both hosts per image.
#[cfg(not(target_arch = "wasm32"))]
static AGENT: std::sync::LazyLock<ureq::Agent> = std::sync::LazyLock::new(ureq::Agent::new_with_defaults);

/// Downloads [`ensure_card_images_with_progress`] keeps in flight. The image
/// host (`cards.scryfall.io`) has no rate limit, and every request to the API
/// itself still waits its turn at [`pace_api_request`], so the workers speed
/// up the image fetches without adding API traffic.
#[cfg(not(target_arch = "wasm32"))]
const DOWNLOAD_WORKERS: usize = 8;

/// Scryfall's hard limit on `/cards/named` and `/cards/search` — the two
/// endpoints a per-card fetch uses — is 2 requests a second (other API
/// methods allow 10). The prefetch used to send them back to back and lean on
/// the 429 backoff, which is how a whole run got rate-limited. Kept for every
/// `api.scryfall.com` request; the image and bulk-data hosts are unlimited.
#[cfg(not(target_arch = "wasm32"))]
const API_REQUEST_GAP: Duration = Duration::from_millis(500);

/// Sleep until [`API_REQUEST_GAP`] has passed since the last API request
/// from this process.
#[cfg(not(target_arch = "wasm32"))]
fn pace_api_request() {
    static LAST: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
    let Ok(mut last) = LAST.lock() else { return };
    if let Some(gap) = last.and_then(|at| API_REQUEST_GAP.checked_sub(at.elapsed())) {
        thread::sleep(gap);
    }
    *last = Some(std::time::Instant::now());
}

/// Percent-encode a card name for use in a Scryfall URL query parameter.
/// Spaces become `+`; all non-ASCII and reserved ASCII characters are encoded.
#[cfg(not(target_arch = "wasm32"))]
fn urlenccode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b' ' => out.push('+'),
            // Unreserved characters (RFC 3986) pass through unchanged.
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            // Everything else (including non-ASCII UTF-8 bytes) is encoded.
            b => {
                let _ = write!(out, "%{b:02X}");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    /// A card the app asks to render jumps the prefetch queue, and each
    /// request is honoured once.
    #[test]
    fn rendering_a_card_jumps_its_download() {
        use super::*;
        // Names no other test asks for — `REQUESTED_IMAGES` is process-wide.
        let pending = [
            CardImage::Front("Prefetch Probe A"),
            CardImage::Front("Prefetch Probe B"),
        ];
        let refs: Vec<&CardImage> = pending.iter().collect();
        note_image_requested(card_filename("Prefetch Probe B"));
        assert_eq!(next_requested(&refs), Some(1));
        assert_eq!(next_requested(&refs), None, "consumed, not re-jumped");
    }

    use super::*;

    #[test]
    fn front_image_filename_is_the_sanitized_name() {
        let spec = CardImage::Front("Lightning Bolt");
        assert_eq!(spec.filename(), "lightning_bolt.jpg");
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
        assert_eq!(spec.filename(), "timbercrown_pathway_back.jpg");
    }

    #[test]
    fn split_card_name_collapses_path_separators() {
        // Split cards ("Wear // Tear", "Reduce // Rubble") embed `/` in
        // their printed names. Without sanitising, `fs::write` interprets
        // `cards/wear_//_tear.jpg` as a nested path and panics with
        // NotFound when the implied parent dirs don't exist.
        assert_eq!(card_filename("Wear // Tear"), "wear____tear.jpg");
        assert_eq!(card_filename("Reduce // Rubble"), "reduce____rubble.jpg");
        assert_eq!(card_back_face_filename("Foo / Bar"), "foo___bar_back.jpg",);
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
        for name in [
            "Bird", "Citizen", "Clue", "Faerie", "Food", "Giant", "Blood", "Treasure",
        ] {
            let spec = CardImage::Token { name };
            assert!(!spec.is_fictional(), "{name} token must not be fictional");
            assert_eq!(spec.filename(), format!("{}.jpg", name.to_lowercase()));
        }
    }

    #[test]
    fn token_label_is_disambiguated_in_logs() {
        let spec = CardImage::Token { name: "Clue" };
        assert_eq!(spec.label(), "Clue token");
    }

    #[test]
    fn ensure_card_images_writes_no_file_for_fictional_cards() {
        use std::fs;
        // Use a temp asset dir so we don't pollute the repo.
        let tmp = std::env::temp_dir().join(format!("crab-scryfall-test-{}", std::process::id(),));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("cards")).expect("temp setup");

        let specs = vec![CardImage::Front("Mount Tyrhus")];
        ensure_card_images(&specs, &tmp);

        // The prefetch no longer stamps placeholder files: the runtime
        // `CardPlaceholderReader` synthesizes them on demand instead.
        let path = tmp.join("cards").join("mount_tyrhus.jpg");
        assert!(
            !path.exists(),
            "fictional card must NOT get a placeholder file on disk: {}",
            path.display(),
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn scryfall_lookup_strips_deck_variant_suffix() {
        // A real card carrying a catalog disambiguator resolves to its true
        // Scryfall name...
        assert_eq!(scryfall_lookup_name("Putrefy (Modern)"), "Putrefy");
        assert_eq!(
            scryfall_lookup_name("Lightning Bolt (Legacy)"),
            "Lightning Bolt"
        );
        // ...while plain names and the synthesized-audit `(bNNN)` suffix are
        // left untouched (those aren't real Scryfall cards).
        assert_eq!(scryfall_lookup_name("Lightning Bolt"), "Lightning Bolt");
        assert_eq!(
            scryfall_lookup_name("Witherbloom Verdance (b202)"),
            "Witherbloom Verdance (b202)"
        );
    }

    #[test]
    fn unavailable_manifest_round_trips() {
        use std::fs;
        let tmp =
            std::env::temp_dir().join(format!("crab-scryfall-unavail-{}", std::process::id(),));
        let cards = tmp.join("cards");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&cards).expect("temp setup");

        // Empty / missing manifest → empty set.
        assert!(load_unavailable(&cards).is_empty());

        mark_unavailable(&cards, "strixhaven_spawner.jpg");
        mark_unavailable(&cards, "lightning_bolt_back.jpg");
        // Re-loading sees both entries, regardless of order.
        let set = load_unavailable(&cards);
        assert!(set.contains("strixhaven_spawner.jpg"));
        assert!(set.contains("lightning_bolt_back.jpg"));
        assert_eq!(set.len(), 2);

        let _ = fs::remove_dir_all(&tmp);
    }

    /// Art cached as PNG before the JPEG switch becomes JPEG in place, its
    /// transparent corners black, and the negative cache follows the rename.
    #[test]
    fn cached_png_art_is_converted_to_jpeg() {
        use std::fs;
        let tmp = std::env::temp_dir().join(format!("crab-scryfall-convert-{}", std::process::id()));
        let cards = tmp.join("cards");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&cards).expect("temp setup");
        // Scryfall's PNG size, with a transparent corner.
        let mut png = image::RgbaImage::from_pixel(745, 1040, image::Rgba([200, 30, 30, 255]));
        for (x, y) in (0..40).flat_map(|x| (0..40).map(move |y| (x, y))) {
            png.put_pixel(x, y, image::Rgba([255, 255, 255, 0]));
        }
        png.save(cards.join("lightning_bolt.png")).unwrap();
        fs::write(cards.join("broken.png"), b"not a png").unwrap();
        fs::write(cards.join(UNAVAILABLE_MANIFEST), "strixhaven_spawner.png\nfoo_back.png\n").unwrap();

        let progress = Progress::default();
        assert_eq!(convert_png_art(&cards, &progress), 1);

        let jpg = image::open(cards.join("lightning_bolt.jpg")).expect("converted").to_rgb8();
        assert_eq!(jpg.dimensions(), (ART_WIDTH, 938), "scaled to Scryfall's `large` width");
        assert!(jpg.get_pixel(0, 0).0.iter().all(|&c| c < 40), "corner laid over black: {:?}", jpg.get_pixel(0, 0));
        assert!(jpg.get_pixel(300, 400).0[0] > 150, "art kept");
        assert!(!cards.join("lightning_bolt.png").exists() && !cards.join("broken.png").exists());
        assert!(!cards.join("broken.jpg").exists(), "an unreadable PNG is dropped and re-downloaded");
        assert_eq!(
            fs::read_to_string(cards.join(UNAVAILABLE_MANIFEST)).unwrap(),
            "strixhaven_spawner.jpg\nfoo_back.jpg\n"
        );
        assert_eq!(*progress.completed.lock().unwrap(), ["cards/lightning_bolt.jpg"]);
        assert_eq!(convert_png_art(&cards, &progress), 0, "nothing left to convert");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn jpeg_bytes_are_stored_as_they_come() {
        let mut jpeg = Vec::new();
        image::DynamicImage::ImageRgb8(image::RgbImage::new(4, 4))
            .write_to(&mut std::io::Cursor::new(&mut jpeg), image::ImageFormat::Jpeg)
            .unwrap();
        assert_eq!(as_jpeg(jpeg.clone()).unwrap(), jpeg);
    }

    /// The bulk file answers what `cards/named?format=image` would have:
    /// a card's one image or its front face's, an MDFC's back face by its
    /// front's name, and never an art-series card or a token sharing a name.
    #[test]
    fn bulk_index_resolves_like_a_named_lookup() {
        let mut index = BulkIndex::default();
        for line in [
            "[",
            r#"{"name":"Brightglass Gearhulk // Brightglass Gearhulk","layout":"art_series","card_faces":[{"name":"Brightglass Gearhulk","image_uris":{"large":"art-card"}},{"name":"Brightglass Gearhulk","image_uris":{"large":"art-card"}}]},"#,
            r#"{"name":"Brightglass Gearhulk","layout":"normal","image_uris":{"large":"gearhulk"}},"#,
            r#"{"name":"Soldier","layout":"token","image_uris":{"large":"soldier-token"}},"#,
            r#"{"name":"Cragcrown Pathway // Timbercrown Pathway","layout":"modal_dfc","card_faces":[{"name":"Cragcrown Pathway","image_uris":{"large":"crag"}},{"name":"Timbercrown Pathway","image_uris":{"large":"timber"}}]},"#,
            r#"{"name":"Gollum, Silent Slinker // Meager Meal","layout":"adventure","image_uris":{"large":"gollum"},"card_faces":[{"name":"Gollum, Silent Slinker"},{"name":"Meager Meal"}]},"#,
            r#"{"name":"Putrefy","layout":"normal","image_uris":{"large":"putrefy"},"unread_field":[1,2]}"#,
            "]",
        ] {
            index.add_line(line).unwrap();
        }
        let url = |spec: CardImage| index.image_url(&spec).map(str::to_owned);
        assert_eq!(url(CardImage::Front("Brightglass Gearhulk")).as_deref(), Some("gearhulk"), "not the art card");
        assert_eq!(url(CardImage::Front("Cragcrown Pathway")).as_deref(), Some("crag"));
        assert_eq!(url(CardImage::Front("Timbercrown Pathway")).as_deref(), Some("crag"), "named by a face: the front");
        assert_eq!(
            url(CardImage::MdfcBack { front: "Cragcrown Pathway", back: "Timbercrown Pathway" }).as_deref(),
            Some("timber")
        );
        assert_eq!(url(CardImage::Front("Meager Meal")).as_deref(), Some("gollum"), "one image for the whole card");
        assert_eq!(url(CardImage::MdfcBack { front: "Putrefy", back: "Nothing" }), None, "no back face: the lookup decides");
        assert_eq!(url(CardImage::Front("PUTREFY (Modern)")).as_deref(), Some("putrefy"), "case and deck suffix");
        assert_eq!(url(CardImage::Front("Soldier")), None, "tokens are not indexed");
        assert_eq!(url(CardImage::Token { name: "Soldier" }), None, "tokens are searched for");
        assert_eq!(url(CardImage::Front("Jokulhaups")), None);
    }

    #[test]
    fn label_disambiguates_back_face_in_logs() {
        let spec = CardImage::MdfcBack {
            front: "Cragcrown Pathway",
            back: "Timbercrown Pathway",
        };
        assert_eq!(
            spec.label(),
            "Timbercrown Pathway (back of Cragcrown Pathway)"
        );
    }
}
