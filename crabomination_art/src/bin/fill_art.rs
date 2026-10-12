//! Fill a browser bundle's asset directory: the client's fonts and cardback,
//! then every catalog card's art from Scryfall into `cards/`. The browser
//! cannot fetch or cache art itself — it loads `assets/cards/<name>.jpg` from
//! the web host and draws a name placeholder for any it does not find — so
//! this is the web host's job, run before serving and again after a deploy.
//!
//!     cargo run -p crabomination_art --bin fill_art -- <assets dir>
//!
//! The same prefetch the native client runs at launch: existing files are
//! skipped and cards Scryfall has no art for are remembered in
//! `cards/.unavailable.txt`, so a rerun fetches only what is new or failed
//! transiently. Interrupting it is safe (each image is written whole).

use std::path::PathBuf;
use std::sync::atomic::Ordering;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [dir] = args.as_slice() else {
        eprintln!("usage: fill_art <assets dir>   (the `assets/` a browser bundle is served from)");
        std::process::exit(2);
    };
    let dir = PathBuf::from(dir);
    crabomination_art::materialize_core_assets(&dir);
    crabomination_art::purge_type_matched_token_art(&dir, crabomination_art::TOKEN_NAMES);
    // The client's walk (`crabomination_art::SpecSet` says why all of it).
    let mut spec_set = crabomination_art::SpecSet::default();
    let demo = crabomination::demo::build_demo_state();
    for player in &demo.players {
        for card in player.library.iter().chain(&player.hand).chain(&player.graveyard) {
            spec_set.visit(&card.definition);
        }
    }
    for factory in crabomination::catalog::all_known_factories() {
        spec_set.visit(&factory());
    }
    let specs = spec_set.into_specs();
    println!("Filling {} ({} images in the catalog)", dir.join("cards").display(), specs.len());

    // Downloads log a line each; the legacy-PNG conversion before them is
    // silent until its summary, so its counters are reported here.
    let progress = crabomination_art::Progress::default();
    let reporter = {
        let progress = progress.clone();
        std::thread::spawn(move || {
            while !progress.finished.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_secs(5));
                if progress.converting.load(Ordering::Relaxed) {
                    let (done, total) = (progress.done.load(Ordering::Relaxed), progress.total.load(Ordering::Relaxed));
                    println!("  converting PNG art to JPEG: {done}/{total}");
                }
            }
        })
    };
    crabomination_art::ensure_card_images_with_progress(&specs, &dir, &progress);
    let _ = reporter.join();
}
