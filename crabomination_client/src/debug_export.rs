//! Game-state export to `<repo>/debug/state-*.json`.
//!
//! Triggered from the in-game HUD's "Export State" button (or the `X` key).
//! Writes the current `ClientView` as pretty-printed JSON so a player can
//! attach a precise board snapshot to a bug report. Each export is paired
//! with a free-form message describing the bug ("Vandalblast targeting
//! my Ornithopter froze the game", etc.) so a maintainer reading the file
//! later has context.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crabomination::game::GameState;
use crabomination::net::ClientView;
use crabomination::snapshot::GameSnapshot;

/// On-disk wrapper that pairs the engine `ClientView` with the user's
/// description of what went wrong. The fields are exposed so engine tests
/// can read back saved states for fixture-based debugging.
///
/// Three levels of fidelity coexist:
///
/// - `view` — always present. Seat-projected `ClientView`, perfect for
///   human inspection (it's what the player saw at the moment of export).
/// - `snapshot` — when present, a schema-stable [`GameSnapshot`] that
///   restores into a playable `GameState` (lossy on triggers).
/// - `full_state` — when present, a complete `GameState` JSON. Bit-exact
///   replay; preserves trigger stack, delayed triggers, continuous
///   effects, pending decisions, and decider state.
#[derive(Serialize, Deserialize)]
pub struct DebugExport {
    /// Free-form text the user typed at export time.
    pub message: String,
    /// Wall-clock seconds since the Unix epoch when the export was taken.
    pub unix_timestamp: u64,
    /// Snapshot of the game from the exporting seat's POV.
    pub view: ClientView,
    /// Schema-stable `GameSnapshot`, when available. Older exports (and
    /// exports from TCP matches where the engine state lives on a peer)
    /// omit this.
    #[serde(default)]
    pub snapshot: Option<GameSnapshot>,
    /// Fully serialized `GameState`, when available. The most faithful
    /// representation; restores into a byte-equivalent engine state.
    #[serde(default)]
    pub full_state: Option<GameState>,
}

/// Resolve the repo root by walking up from the client crate's manifest dir
/// (`CARGO_MANIFEST_DIR`) until we find a directory containing `Cargo.lock`
/// (the workspace root marker for this project). Falls back to the current
/// working directory if the walk fails.
fn repo_root() -> PathBuf {
    let start = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut cur: &Path = start;
    loop {
        if cur.join("Cargo.lock").exists() {
            return cur.to_path_buf();
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => return std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}

/// Write `view` + `message` (+ optional engine snapshots) to
/// `<repo>/debug/state-<turn>-<step>-<unix>.json`. Returns the absolute
/// file path on success.
pub fn export_full(
    view: &ClientView,
    message: &str,
    snapshot: Option<GameSnapshot>,
    full_state: Option<GameState>,
) -> Result<PathBuf, String> {
    write_export(view, message, snapshot, full_state)
}

/// Backward-compatible wrapper that exports just the seat-projected view
/// (no full engine snapshot). Used by network-game exports where the
/// client doesn't have direct access to the authoritative state.
#[allow(dead_code)]
pub fn export_client_view(view: &ClientView, message: &str) -> Result<PathBuf, String> {
    write_export(view, message, None, None)
}

fn write_export(
    view: &ClientView,
    message: &str,
    snapshot: Option<GameSnapshot>,
    full_state: Option<GameState>,
) -> Result<PathBuf, String> {
    let dir = repo_root().join("debug");
    fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let unix = now.as_secs();
    // Nanosecond fragment disambiguates within the same wall-clock second
    // — needed because `cargo test` runs tests in parallel and they all
    // write into the same `<repo>/debug` dir.
    let nanos = now.subsec_nanos();
    let step = format!("{:?}", view.step).to_lowercase();
    let path = dir.join(format!("state-t{}-{step}-{unix}-{nanos:09}.json", view.turn));

    let payload = DebugExport {
        message: message.to_string(),
        unix_timestamp: unix,
        view: view.clone(),
        snapshot,
        full_state,
    };
    let json = serde_json::to_string_pretty(&payload)
        .map_err(|e| format!("serialize DebugExport: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(path)
}

/// Read back a previously exported state file. Used by the client's
/// `--load-state` flag and by the engine test suite as a fixture loader
/// for replay-based regression tests.
pub fn load_debug_export(path: &Path) -> Result<DebugExport, String> {
    let body = fs::read_to_string(path)
        .map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str::<DebugExport>(&body)
        .map_err(|e| format!("parse {}: {e}", path.display()))
}

/// List every `state-*.json` file currently in the repo's `debug/` dir,
/// sorted newest-first by mtime. Drives the "Load Debug State" picker.
pub fn list_exports() -> Vec<PathBuf> {
    let dir = repo_root().join("debug");
    let Ok(read) = fs::read_dir(&dir) else { return Vec::new() };
    let mut entries: Vec<(PathBuf, SystemTime)> = read
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) != Some("json") {
                return None;
            }
            let mtime = e.metadata().and_then(|m| m.modified()).ok()?;
            Some((p, mtime))
        })
        .collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.1));
    entries.into_iter().map(|(p, _)| p).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination::game::TurnStep;
    use crabomination::mana::ManaPool;
    use crabomination::net::{ClientView, LibraryView, PlayerView};

    fn empty_view() -> ClientView {
        ClientView {
            your_seat: 0,
            active_player: 0,
            priority: 0,
            step: TurnStep::PreCombatMain,
            turn: 1,
            extra_phase: false,
            attackable_players: Vec::new(),
            permanents_to_graveyard_this_turn: 0,
            damage_rewritten_this_turn: None,
            players: vec![PlayerView {
                locked_cast_colors: vec![],
                locked_cast_kinds: vec![],
                uncounterable_next: vec![],
                draws_stolen_by: None,
                prevention_remaining: None,
                prevention_source_colors: Vec::new(),
                prevention_next_instances: 0,
                prevention_source_names: Vec::new(),
                seat: 0,
                name: "Alice".into(),
                life: 20,
                starting_life: 20,
                poison_counters: 0,
                poison_loss_threshold: 10,
                phased_out: vec![],
                energy: 0,
                experience: 0,
                rad_counters: 0,
                mana_pool: ManaPool::default(),
                kept_mana: 0,
                library: LibraryView::default(),
                graveyard: vec![],
                hand: vec![],
                lands_played_this_turn: 0,
                land_plays_remaining: 1,
                extra_loyalty_activations: 0,
                skip_next_combat: 0,
                skip_turns: 0,
                untargetable: false,
                controlled_by: None,
                first_spell_tax_charges: 0,
                life_gained_this_turn: 0,
                cards_drawn_this_turn: 0,
                draw_cap: None,
                cards_left_graveyard_this_turn: 0,
                creatures_died_this_turn: 0,
                next_creature_bonus_counters: 0,
                next_creature_gains_haste: false,
                cards_exiled_this_turn: 0,
                instants_or_sorceries_cast_this_turn: 0,
                creatures_cast_this_turn: 0,
                spells_cast_this_turn: 0,
                spell_cast_lock: Default::default(),
                even_mv_cast_locked: false,
                development_locked: false,
                max_hand_size: Some(7),
                command: vec![],
                commanders: vec![],
                commander_casts: vec![],
                emblems: vec![],
                eliminated: false,
                loss_reason: None,
                has_prevention_shield: false,
                damage_fully_prevented: false,
                damage_redirect_to: None,
                devotion: [0; 5],
                hand_revealed_to_viewer: false,
                is_monarch: false,
                has_city_blessing: false,
                enchanted_by: Vec::new(),
                cannot_gain_life: false,
                cant_cast_noncreature: false,
                arbiter_cant_cast: false,
                arbiter_cant_attack: false,
                life_locked: false,
                has_hexproof: false,
                commander_damage_taken: vec![],
                team: 0,
                dungeon: None,
                dungeons_completed: 0,
                coven_active: false,
                descend_count: 0,
                descended_this_turn_count: 0,
                committed_crime_this_turn: false,
                ring_temptations: 0,
                ring_bearer: None,
                void_active: false,
                threshold_active: false,
                metalcraft_active: false,
                ferocious_active: false,
                hellbent_active: false,
                formidable_active: false,
                mazes_end_gate_progress: None,
                speed: 0,
                at_max_speed: false,
            }],
            battlefield: vec![],
            stack: vec![],
            pending_decision: None,
            exile: vec![],
            game_over: None,
            damage_cant_be_prevented_this_turn: false,
            spend_mana_as_any_color: false,
            combat_damage_prevented_this_turn: false,
            permanents_enter_tapped_this_turn: false,
            shared_team_turns: false,
            attack_tax_this_turn: 0,
            turn_effects: Vec::new(),
            block_tax_this_turn: 0,
            combat_chooser: None,
            max_attackers_per_combat: None,
            max_blockers_per_combat: None,
            face_down_cast_cost: 3,
            day_night: None,
            combat_preview: None,
            castable_hand: vec![],
            back_castable_hand: vec![],
            prepare_castable: vec![],
            // The printed {3} morph price — the neutral value a stub view
            // with no cost-reduction statics projects.
            face_down_cast_cost: 3,
            spliceable_hand: vec![],
            pitchable_hand: vec![],
            kickable_hand: vec![],
            kicker_option_sets: vec![],
            buyback_hand: vec![],
            bestowable_hand: vec![],
            miracle_hand: vec![],
            bargainable_hand: vec![],
            squadable_hand: vec![],
            spreeable_hand: vec![],
            replicatable_hand: vec![],
            conspirable_hand: vec![],
            multikickable_hand: vec![],
            convokable_hand: vec![],
            dashable_hand: vec![],
            blitzable_hand: vec![],
            warpable_hand: vec![],
            suspendable_hand: vec![],
            foretellable_hand: vec![],
            plottable_hand: vec![],
            castable_plotted: vec![],
            adventurable_hand: vec![],
            omenable_hand: vec![],
            splittable_right_hand: vec![],
            prototypable_hand: vec![],
            room_castable_hand: vec![],
            room_unlockable: vec![],
            activatable_permanents: vec![],
            hand_activatable: vec![],
            morphable_hand: vec![],
            turn_up_able: vec![],
            reinforceable_hand: vec![],
            discard_activatable_hand: vec![],
            legal_attackers: vec![],
            legal_blockers: vec![],
            attack_bands: vec![],
        }
    }

    #[test]
    fn export_writes_a_readable_file() {
        let cv = empty_view();
        let path = export_client_view(&cv, "smoke test").expect("export should succeed");
        assert!(path.exists(), "exported file must exist on disk");
        let body = fs::read_to_string(&path).expect("read back");
        assert!(body.contains("\"turn\": 1"), "json should contain turn field: {body}");
        assert!(body.contains("\"name\": \"Alice\""), "json should contain player name");
        assert!(body.contains("\"message\": \"smoke test\""),
            "json should embed the user's message: {body}");
        // Clean up so repeated test runs don't leave noise behind.
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn export_then_load_round_trips() {
        let cv = empty_view();
        let path = export_client_view(&cv, "round-trip").expect("export should succeed");
        let parsed = load_debug_export(&path).expect("load should succeed");
        assert_eq!(parsed.message, "round-trip");
        assert_eq!(parsed.view.turn, 1);
        assert_eq!(parsed.view.players.len(), 1);
        assert_eq!(parsed.view.players[0].name, "Alice");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn export_with_full_state_round_trips_lossless() {
        use crabomination::catalog;
        use crabomination::effect::Effect;
        use crabomination::game::{GameAction, StackItem, TurnStep};
        use crabomination::player::Player;

        // Build a state with a Trigger on the stack — the snapshot path
        // would drop this; the full-state path must preserve it.
        let mut g = crabomination::game::GameState::new(vec![
            Player::new(0, "Alice"),
            Player::new(1, "Bob"),
        ]);
        g.step = TurnStep::PreCombatMain;
        let forest = g.add_card_to_hand(0, catalog::forest());
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.stack.push(
            crabomination::game::types::TriggerPush::new(
                crabomination::card::CardId(99),
                0,
                Effect::Noop,
            )
            .build(),
        );

        let cv = empty_view();
        let path = export_full(&cv, "full state RT", None, Some(g)).expect("export_full");
        let loaded = load_debug_export(&path).expect("load");
        assert!(loaded.full_state.is_some(), "full GameState must round-trip");
        let mut restored = loaded.full_state.unwrap();
        // Trigger preserved (would have been dropped in the snapshot path).
        assert_eq!(restored.stack.len(), 1);
        assert!(matches!(restored.stack[0], StackItem::Trigger { .. }));
        // And once the stack is clear the restored state still drives
        // the engine: cast actions are accepted, mana works, etc.
        restored.stack.clear();
        restored
            .perform_action(GameAction::PlayLand(forest))
            .expect("PlayLand on the restored full state");
        assert!(restored.battlefield.iter().any(|c| c.id == forest));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn export_with_snapshot_round_trips_to_a_playable_state() {
        use crabomination::catalog;
        use crabomination::game::{GameAction, TurnStep};
        use crabomination::player::Player;
        use crabomination::snapshot::GameSnapshot;

        // Build a minimal game state, snapshot it, write it to disk via
        // export_full, read it back, restore the snapshot, and exercise
        // an action against the restored state. This is the end-to-end
        // path the in-game "Export → Load" workflow takes.
        let mut g = crabomination::game::GameState::new(vec![
            Player::new(0, "Alice"),
            Player::new(1, "Bob"),
        ]);
        g.step = TurnStep::PreCombatMain;
        let forest = g.add_card_to_hand(0, catalog::forest());
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;

        let cv = empty_view();
        let snap = GameSnapshot::capture(&g);
        let path = export_full(&cv, "snapshot RT", Some(snap), None).expect("export_full");
        let loaded = load_debug_export(&path).expect("load");
        assert!(loaded.snapshot.is_some(), "snapshot must round-trip in JSON");
        let mut restored = loaded.snapshot.unwrap().restore().expect("restore");
        restored
            .perform_action(GameAction::PlayLand(forest))
            .expect("PlayLand on restored state");
        assert!(restored.battlefield.iter().any(|c| c.id == forest));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn list_exports_returns_existing_files() {
        let cv = empty_view();
        let path = export_client_view(&cv, "listed").expect("export should succeed");
        let listed = list_exports();
        assert!(
            listed.iter().any(|p| p == &path),
            "list_exports should include the file we just wrote: {listed:?}",
        );
        let _ = fs::remove_file(&path);
    }
}
