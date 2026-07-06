//! Tiny persistent key-value storage, portable across native and browser.
//!
//! Native: one file per key under the OS config dir (same location
//! `config.rs` already uses). Browser (wasm32): `window.localStorage`,
//! which survives reloads and browser restarts (~5 MB quota, plenty for
//! config + resume tokens).
//!
//! Values are strings; callers serialize structure (TOML/JSON) themselves.

// Native config still talks to its long-standing file paths directly; this
// backend exists for parity and future native use (resume tokens etc.).
#[allow(dead_code)]
#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::path::PathBuf;

    fn key_path(key: &str) -> Option<PathBuf> {
        let dir = dirs::config_dir()?.join("crabomination");
        Some(dir.join(format!("{key}.storage")))
    }

    pub fn load(key: &str) -> Option<String> {
        std::fs::read_to_string(key_path(key)?).ok()
    }

    pub fn save(key: &str, value: &str) -> bool {
        let Some(path) = key_path(key) else { return false };
        if let Some(parent) = path.parent()
            && std::fs::create_dir_all(parent).is_err()
        {
            return false;
        }
        std::fs::write(&path, value).is_ok()
    }

    pub fn remove(key: &str) {
        if let Some(path) = key_path(key) {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[allow(dead_code)]
#[cfg(target_arch = "wasm32")]
mod imp {
    fn local_storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    fn namespaced(key: &str) -> String {
        format!("crabomination.{key}")
    }

    pub fn load(key: &str) -> Option<String> {
        local_storage()?.get_item(&namespaced(key)).ok().flatten()
    }

    pub fn save(key: &str, value: &str) -> bool {
        local_storage()
            .map(|s| s.set_item(&namespaced(key), value).is_ok())
            .unwrap_or(false)
    }

    pub fn remove(key: &str) {
        if let Some(s) = local_storage() {
            let _ = s.remove_item(&namespaced(key));
        }
    }
}

#[allow(unused_imports)]
pub use imp::{load, remove, save};
