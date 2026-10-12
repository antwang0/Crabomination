//! The client's half of card art: the asset paths the app loads, a
//! placeholder for any card image that is not on disk, and the hot reload
//! that swaps real art in as the prefetch lands it. The prefetch itself —
//! which images, their filenames, fetching them from Scryfall — is
//! `crabomination_art`, shared with the server.

use std::path::Path;
use std::sync::Arc;

use ab_glyph::FontVec;
use bevy::asset::io::{
    AssetReader, AssetReaderError, ErasedAssetReader, PathStream, Reader, VecReader,
};
use crabomination_art::{card_back_face_filename, card_filename, note_image_requested};

/// The prefetch's progress ([`crabomination_art::Progress`]) as a resource:
/// the menu reads its counters and [`reload_completed_images`] drains the
/// images it has written.
#[derive(bevy::prelude::Resource, bevy::prelude::Deref, Clone, Default)]
pub struct ImagePrefetch(pub crabomination_art::Progress);

/// Drain freshly downloaded image paths and hot-reload them so any
/// already-spawned placeholder texture swaps to the real art.
pub fn reload_completed_images(
    prefetch: bevy::prelude::Res<ImagePrefetch>,
    asset_server: bevy::prelude::Res<bevy::asset::AssetServer>,
    mut drained_after_finish: bevy::prelude::Local<bool>,
) {
    // Stop taking the mutex every frame forever once the prefetch thread is
    // done: after `finished` is observed, do one final drain and go inert.
    if *drained_after_finish {
        return;
    }
    if prefetch.finished.load(std::sync::atomic::Ordering::Relaxed) {
        *drained_after_finish = true;
    }
    let Ok(mut completed) = prefetch.completed.lock() else {
        return;
    };
    for path in completed.drain(..) {
        asset_server.reload(path);
    }
}

/// Generated placeholder dimensions: the 63 × 88 card proportion
/// (`card::proxy`).
#[cfg(test)]
const PLACEHOLDER_W: u32 = crate::card::proxy::PROXY_W;
#[cfg(test)]
const PLACEHOLDER_H: u32 = crate::card::proxy::PROXY_H;

/// Load the UI font for placeholder text. Returns `None` if the font file
/// isn't where we expect — the placeholder then renders as a blank white
/// card. Public so the asset-source registration in `main` can load it
/// once and share it with [`CardPlaceholderReader`].
pub fn load_placeholder_font(assets_dir: &Path) -> Option<FontVec> {
    #[cfg(not(target_arch = "wasm32"))]
    let bytes = std::fs::read(assets_dir.join(crate::theme::FONT_PATH)).ok()?;
    // The browser has no filesystem to read it from (the `fs::read` failed
    // and every placeholder was a blank frame), and the asset server's copy
    // is fetched asynchronously, after the reader is built. 65 KB embedded.
    #[cfg(target_arch = "wasm32")]
    let bytes = {
        let _ = assets_dir;
        include_bytes!("../assets/fonts/MiranoExtendedFreebie-Light.ttf").to_vec()
    };
    FontVec::try_from_vec(bytes).ok()
}

/// Encode a drawn face (`card::proxy`) in `format`. Used by
/// [`CardPlaceholderReader`] to serve a generated card image for a path that
/// has no file on disk; the asset loader decodes by the path's extension, so
/// the bytes must match it.
fn proxy_bytes(face: &crate::card::proxy::ProxyFace, font: Option<&FontVec>, format: image::ImageFormat) -> Vec<u8> {
    let img = image::DynamicImage::ImageRgba8(crate::card::proxy::render_proxy(face, font));
    // JPEG has no alpha channel; the faces are opaque anyway.
    let img = if format == image::ImageFormat::Jpeg { image::DynamicImage::ImageRgb8(img.to_rgb8()) } else { img };
    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), format).expect("encode placeholder to memory");
    buf
}

/// The name-only placeholder for an art-less card: a drawn face that knows
/// just the name (the reader has only the path to go on).
fn placeholder_bytes(name: &str, font: Option<&FontVec>) -> Vec<u8> {
    proxy_bytes(&crate::card::proxy::ProxyFace::named(name), font, image::ImageFormat::Jpeg)
}

/// The face a `cards/proxy_….png` path encodes (`card::proxy`), if it is one.
fn proxy_face_from_asset_path(path: &Path) -> Option<crate::card::proxy::ProxyFace> {
    if path.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()) != Some("cards") {
        return None;
    }
    crate::card::proxy::parse_proxy_stem(path.file_stem()?.to_str()?)
}

/// Recover a readable card name from a `cards/<sanitized>.jpg` asset path,
/// for placeholder text. Reversing `sanitize_name` is lossy (we can't
/// restore exact capitalization or punctuation), so we just title-case the
/// de-underscored stem — fine for a placeholder. Returns `None` for any
/// path that isn't a card image, so the reader only ever synthesizes card
/// art (never fonts / models / the cardback).
fn card_name_from_asset_path(path: &Path) -> Option<String> {
    if path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        != Some("cards")
    {
        return None;
    }
    let stem = path.file_stem()?.to_str()?;
    let base = stem.strip_suffix("_back").unwrap_or(stem);
    let name = base
        .split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    (!name.is_empty()).then_some(name)
}

/// Wraps the platform's default file [`AssetReader`] and, for any missing
/// `cards/<name>.jpg`, synthesizes a white name-placeholder on the fly
/// instead of failing — so no placeholder files live on disk. A drawn proxy
/// face (`cards/proxy_….png`) is served the same way, as a PNG.
///
/// The prefetch now runs on a background thread, so a missing card image
/// is either genuinely art-less (a synthesized card or a 404) or simply
/// not downloaded *yet* — both get the name placeholder, and
/// `reload_completed_images` hot-swaps in the real art as downloads land.
/// Any non-card missing path falls through to the inner reader's
/// `NotFound`.
pub struct CardPlaceholderReader {
    inner: Box<dyn ErasedAssetReader>,
    font: Arc<Option<FontVec>>,
}

impl CardPlaceholderReader {
    pub fn new(inner: Box<dyn ErasedAssetReader>, font: Arc<Option<FontVec>>) -> Self {
        Self { inner, font }
    }
}

impl AssetReader for CardPlaceholderReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        match self.inner.read(path).await {
            Ok(reader) => Ok(reader),
            Err(AssetReaderError::NotFound(_)) => {
                let font = self.font.as_ref().as_ref();
                // A drawn token face first (its path encodes the face), then
                // any other missing card image by its name.
                let bytes = match proxy_face_from_asset_path(path) {
                    Some(face) => Some(proxy_bytes(&face, font, image::ImageFormat::Png)),
                    None => card_name_from_asset_path(path).map(|name| placeholder_bytes(&name, font)),
                };
                match bytes {
                    Some(bytes) => Ok(Box::new(VecReader::new(bytes)) as Box<dyn Reader + 'a>),
                    None => Err(AssetReaderError::NotFound(path.to_path_buf())),
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        self.inner.read_meta(path).await
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        self.inner.read_directory(path).await
    }

    async fn is_directory<'a>(&'a self, path: &'a Path) -> Result<bool, AssetReaderError> {
        self.inner.is_directory(path).await
    }
}

/// The asset directory card art is downloaded into (`<dir>/cards/…`), for
/// asking whether a card's real art is on disk.
#[derive(bevy::prelude::Resource, Clone)]
pub struct CardArtDir(pub std::path::PathBuf);

impl CardArtDir {
    /// Whether real art for `name` has been downloaded.
    pub fn has_art(&self, name: &str) -> bool {
        self.0.join("cards").join(card_filename(name)).is_file()
    }
}

/// Asset path relative to the assets/ root, for use with Bevy's AssetServer.
pub fn card_asset_path(name: &str) -> String {
    let file = card_filename(name);
    note_image_requested(file.clone());
    format!("cards/{file}")
}

/// Asset path for an MDFC back-face image.
pub fn card_back_face_asset_path(name: &str) -> String {
    let file = card_back_face_filename(name);
    note_image_requested(file.clone());
    format!("cards/{file}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The paths the app loads are `cards/` plus the art crate's filenames,
    /// which is where the prefetch writes them.
    #[test]
    fn card_asset_paths_name_the_prefetched_files() {
        assert_eq!(card_asset_path("Lightning Bolt"), "cards/lightning_bolt.jpg");
        assert_eq!(card_asset_path("Reduce // Rubble"), "cards/reduce____rubble.jpg");
        assert_eq!(card_back_face_asset_path("Timbercrown Pathway"), "cards/timbercrown_pathway_back.jpg");
    }

    #[test]
    fn placeholder_bytes_are_a_card_proportioned_jpeg() {
        // The reader serves these bytes for a missing card image, whose path
        // ends `.jpg`. No font here → blank white card, still decodable.
        let bytes = placeholder_bytes("Awesome Presentation", None);
        assert_eq!(image::guess_format(&bytes).unwrap(), image::ImageFormat::Jpeg);
        let img = image::load_from_memory(&bytes).expect("placeholder must decode");
        assert_eq!((img.width(), img.height()), (PLACEHOLDER_W, PLACEHOLDER_H));
    }

    #[test]
    fn card_name_recovered_from_asset_path() {
        use std::path::Path;
        assert_eq!(
            card_name_from_asset_path(Path::new("cards/awesome_presentation.jpg")).as_deref(),
            Some("Awesome Presentation"),
        );
        // Back-face suffix is stripped; non-card paths are ignored.
        assert_eq!(
            card_name_from_asset_path(Path::new("cards/searstep_pathway_back.jpg")).as_deref(),
            Some("Searstep Pathway"),
        );
        assert_eq!(card_name_from_asset_path(Path::new("fonts/ui.ttf")), None);
        assert_eq!(card_name_from_asset_path(Path::new("cardback.png")), None);
    }

    #[test]
    fn reader_synthesizes_placeholder_for_missing_card_but_not_other_paths() {
        use bevy::asset::AsyncReadExt;
        use bevy::asset::io::{
            AssetReader, AssetReaderError, ErasedAssetReader, PathStream, Reader,
        };
        use bevy::tasks::block_on;
        use std::path::Path;

        // Inner reader that reports every path as missing.
        struct AlwaysMissing;
        impl AssetReader for AlwaysMissing {
            async fn read<'a>(
                &'a self,
                path: &'a Path,
            ) -> Result<impl Reader + 'a, AssetReaderError> {
                Err::<bevy::asset::io::VecReader, _>(AssetReaderError::NotFound(path.to_path_buf()))
            }
            async fn read_meta<'a>(
                &'a self,
                path: &'a Path,
            ) -> Result<impl Reader + 'a, AssetReaderError> {
                Err::<bevy::asset::io::VecReader, _>(AssetReaderError::NotFound(path.to_path_buf()))
            }
            async fn read_directory<'a>(
                &'a self,
                path: &'a Path,
            ) -> Result<Box<PathStream>, AssetReaderError> {
                Err(AssetReaderError::NotFound(path.to_path_buf()))
            }
            async fn is_directory<'a>(&'a self, _path: &'a Path) -> Result<bool, AssetReaderError> {
                Ok(false)
            }
        }

        let reader = CardPlaceholderReader::new(
            Box::new(AlwaysMissing) as Box<dyn ErasedAssetReader>,
            std::sync::Arc::new(None),
        );

        // Missing card image → a synthesized placeholder is served, in the
        // format the path's extension tells the loader to expect.
        let mut r = block_on(AssetReader::read(&reader, Path::new("cards/test_card.jpg")))
            .expect("missing card image must be served a placeholder");
        let mut bytes = Vec::new();
        block_on(AsyncReadExt::read_to_end(&mut r, &mut bytes)).unwrap();
        assert_eq!(image::guess_format(&bytes).unwrap(), image::ImageFormat::Jpeg);
        let img = image::load_from_memory(&bytes).expect("served bytes must decode");
        assert_eq!((img.width(), img.height()), (PLACEHOLDER_W, PLACEHOLDER_H));
        // A drawn proxy face keeps its `.png` path and is served as a PNG.
        let proxy = crate::card::proxy::proxy_asset_path(&crate::card::proxy::ProxyFace::named("Goblin"));
        let mut r = block_on(AssetReader::read(&reader, Path::new(&proxy))).expect("proxy served");
        let mut bytes = Vec::new();
        block_on(AsyncReadExt::read_to_end(&mut r, &mut bytes)).unwrap();
        assert_eq!(image::guess_format(&bytes).unwrap(), image::ImageFormat::Png);

        // A missing non-card path is NOT synthesized — it still 404s.
        assert!(block_on(AssetReader::read(&reader, Path::new("fonts/ui.ttf"))).is_err());
    }
}

