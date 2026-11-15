//! The local bots' pilot and the nets under it: which bot a local seat
//! gets ([`local_bot`]), the Hint's bot ([`hint_bot`]), the honest redeal
//! switch ([`honest_for`]), and the once-per-process net loads. Split out
//! of `menu.rs` (2,900 lines), which keeps the menu and the match spawners.

use super::*;

/// The bot a local seat gets: the lobby's pilot — the 256-iteration
/// search on the adopted default with the champion value net as its leaf
/// (round 74, 2026-09-15: 53.3 / 55.2 over the material leaf inside the
/// search, `server::lobby::default_bot` has the record) — so single-player
/// and hosted games face the same opponent. The net is loaded here on
/// first use from `CRAB_NET` or `nets/champion.safetensors`; a checkout
/// without the file plays the material leaf (the round-64 pilot, 55.25
/// over the heuristic default) and says so once on stderr, the same
/// fallback the server has. Before round 65 the client built its own
/// pilot — `net_eval_det1` at 64 iterations without the combat chains —
/// and forty September replays showed its attack judgment two generations
/// behind the lobby's (ML_NOTES "Round 65").
///
/// A pod (more than two seats) gets the heuristic default, as the lobby's
/// does: the net reads only two-seat games, and the search on the material
/// leaf is unmeasured in a pod.
///
/// An FRA game plays with the FRA bot fixes ([`EvalWeights::with_fra_fixes`]:
/// burn at creatures and for lethal, no X=0 Hydra, trades and free attacks,
/// tricks with riders, counters in their counter mode, ...): this pilot with
/// them beat it without them 56.0 [54.4, 57.5] / 55.9 [54.3, 57.4] (FRA sealed, 200 x 12,
/// ladder seeds 43 / 97, `.ladder/fra_next`). A Sealed or Cube match
/// (`limited`) plays with them too, on the champion leaf (round 85,
/// `.ladder/r85`): SOS sealed 53.5 [52.2, 54.9] / 53.9 [52.6, 55.2], cube
/// 54.1 [52.6, 55.7] / 53.3 [51.7, 54.9] (seeds 43 / 97). Modern and the
/// college decks are unmeasured and stay off. `CRAB_FRA_FIXES=0` turns them
/// off; `CRAB_FRA_FIXES=1` turns them on in every game.
///
/// An FRA game (`fra`, from `fra_mode::is_fra_game`) puts the Reality
/// Fracture net under the search instead — `CRAB_FRA_NET` or
/// `nets/fra-champion.safetensors`, loaded into its own slot. The champion
/// is the SOS net: on FRA it adds nothing over the material leaf, and the
/// FRA net makes the same search +3.75 (round 82, `ML_NOTES`). Since round
/// 86 the file is that round's rated-deck seed-43 net (pinned 0.274 learner
/// steps a game): +1.8 scored and +0.9 under this search over round 82's,
/// on ladder seeds neither was chosen on.
///
/// Since 2026-11-02 the same games (`fra || limited`) search their main
/// phases and combat over the complete move list (`MctsConfig::
/// menu_windows`) on the priors of the phase-5 policy net
/// (`nets/policy-champion.safetensors`, gen 3 of the expert-iteration loop;
/// `CRAB_POLICY_NET` overrides it, `CRAB_MENU_SEARCH=0/1` turns the search
/// off / on everywhere). Against this pilot without it, paired 120 x 12:
/// SOS sealed 58.0 [56.0, 59.9] / 55.8 [53.9, 57.8], FRA sealed (FRA leaf)
/// 57.5 [55.3, 59.7] / 57.6 [55.5, 59.7], cube 54.0 [51.7, 56.2] / 54.1 [51.7, 56.5]
/// (seeds 43 / 97, `.ladder/p5`, `.ladder/adopt`). The rollout budget is
/// the same 256, split across the same worker threads. Since 2026-11-10 that
/// search plays combat with its fix (`MctsConfig::with_combat_fix`: FRA
/// sealed +2.45, the blunder set 35 / 35 from 23.6), and since 2026-11-12
/// with the search fixes (`MctsConfig::with_search_fixes`: NO LOSS on SOS
/// sealed, the 60-item blunder set 60 / 60 from 57.0).
pub(crate) fn local_bot(n_seats: usize, fra: bool, limited: bool) -> Box<dyn crabomination::server::Bot> {
    if n_seats > 2 {
        return Box::new(HeuristicBot::new());
    }
    load_champion_once();
    let mut weights = crabomination::server::EvalWeights::net_on_default();
    if fra && load_fra_net_once() {
        weights.net_slot = crabomination::server::net_eval::SLOT_FRA;
    }
    let fixes = match std::env::var_os("CRAB_FRA_FIXES") {
        Some(v) => v != "0",
        None => fra || limited,
    };
    if fixes {
        weights = weights.with_fra_fixes();
        static SAID: std::sync::Once = std::sync::Once::new();
        SAID.call_once(|| eprintln!("the local bot plays with the FRA fixes bundle (CRAB_FRA_FIXES=0 turns it off)"));
    }
    if limited && HONEST.load(std::sync::atomic::Ordering::Relaxed) {
        weights = weights.honest();
    }
    let menu = match std::env::var_os("CRAB_MENU_SEARCH") {
        Some(v) => v != "0",
        None => fra || limited,
    } && load_policy_net_once();
    let (menu_windows, policy_slot) = if menu {
        use crabomination::server::mcts::{MENU_COMBAT, MENU_MAIN};
        (MENU_MAIN | MENU_COMBAT, crabomination::server::net_eval::SLOT_CANDIDATE)
    } else {
        (0, 0)
    };
    let cfg = crabomination::server::MctsConfig {
        iterations: 256,
        horizon_turns: 3,
        weights,
        search_threads: bot_search_threads(),
        menu_windows,
        policy_slot,
        ..crabomination::server::MctsConfig::default()
    };
    Box::new(crabomination::server::MctsBot::new(if menu { cfg.with_combat_fix().with_search_fixes() } else { cfg }))
}

/// The bot a Hint asks (`server::hint`): the local pilot for the state's
/// table — so a hint is as strong as the bots you are playing.
pub(crate) fn hint_bot(state: &GameState) -> Box<dyn crabomination::server::Bot> {
    let fra = crabomination::fra_mode::is_fra_game(state);
    local_bot(state.players.len(), fra, fra)
}

/// Whether this match's local bots redeal honestly; set by [`honest_for`].
pub(super) static HONEST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// `CRAB_HONEST=1`, off by default: in Sealed and Cube the local bot's
/// search stops reading your real decklist — it deals your hidden cards
/// from the format's card pool by the table net's belief head
/// (`CRAB_TABLE_NET`; `server::honest`) instead of reshuffling your actual
/// hand and library. Unmeasured, so not the default. Call before the
/// match's [`local_occupants`]: it pins the pool to `format`'s (cube's, or
/// the sealed set the game is in) and arms [`local_bot`].
pub(crate) fn honest_for(format: MatchFormat, state: &GameState) {
    use crabomination::server::net_eval;
    use crabomination::table_selfplay::{Mode, mode_pool, pool_for};
    let on = format.is_limited() && std::env::var_os("CRAB_HONEST").is_some_and(|v| v != "0") && load_table_net_once();
    if on {
        let vocab = net_eval::vocab();
        net_eval::pin_table_pool(Some(if format == MatchFormat::Cube { mode_pool(Mode::Cube, vocab) } else { pool_for(state, vocab) }));
    }
    HONEST.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// Load `CRAB_TABLE_NET` into the table slot, once per process; whether it
/// is there with the belief head the honest redeal reads.
pub(super) fn load_table_net_once() -> bool {
    use crabomination::server::net_eval;
    static LOADED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LOADED.get_or_init(|| {
        let Ok(path) = std::env::var("CRAB_TABLE_NET") else {
            eprintln!("CRAB_HONEST needs CRAB_TABLE_NET=<table net>; the local bot keeps the old redeal");
            return false;
        };
        match net_eval::load_table(std::path::Path::new(&path), None) {
            Ok(()) if net_eval::table_has_belief() => {
                eprintln!("table net loaded from {path}; the local bot deals your hidden cards from the pool");
                true
            }
            Ok(()) => {
                eprintln!("table net {path} carries no belief head; the local bot keeps the old redeal");
                false
            }
            Err(e) => {
                eprintln!("table net {path}: {e}; the local bot keeps the old redeal");
                false
            }
        }
    })
}

/// Load the policy net — the menu search's priors — into the candidate
/// slot, once per process; whether it is there. Missing or bad: announced,
/// and the bot keeps the six-cast search.
pub(super) fn load_policy_net_once() -> bool {
    use crabomination::server::net_eval;
    static LOADED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LOADED.get_or_init(|| {
        let path = std::env::var("CRAB_POLICY_NET").unwrap_or_else(|_| "nets/policy-champion.safetensors".to_string());
        match net_eval::load_slot(net_eval::SLOT_CANDIDATE, std::path::Path::new(&path)) {
            Ok(()) if net_eval::slot_has_policy(net_eval::SLOT_CANDIDATE) => {
                eprintln!("policy net loaded from {path}; the local bot searches main phases and combat over every legal move");
                true
            }
            Ok(()) => {
                eprintln!("policy net {path} carries no policy head; the local bot keeps the six-cast search");
                false
            }
            Err(e) => {
                eprintln!("policy net {path}: {e}; the local bot keeps the six-cast search");
                false
            }
        }
    })
}

/// Load the champion value net into the bot's slot, once per process.
/// Missing file: the material leaf, announced. Bad file: announced and
/// the material leaf — a game must still start, unlike the server, whose
/// boot refuses a bad net because it advertises the bot's strength.
pub(super) fn load_champion_once() {
    use crabomination::server::net_eval;
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let path = std::env::var("CRAB_NET").unwrap_or_else(|_| "nets/champion.safetensors".to_string());
        let p = std::path::Path::new(&path);
        if !p.exists() {
            eprintln!("value net {path} not found; the local bot searches on the material leaf");
            return;
        }
        match net_eval::load_slot(net_eval::SLOT_BEST, p) {
            Ok(()) => eprintln!("value net loaded from {path}; the local bot searches on its leaf"),
            Err(e) => eprintln!("value net {path}: {e}; the local bot searches on the material leaf"),
        }
    });
}

/// Load the FRA value net into its slot, once per process; whether it is
/// there. Missing or bad: announced, and the FRA game keeps the champion.
pub(super) fn load_fra_net_once() -> bool {
    use crabomination::server::net_eval;
    static LOADED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LOADED.get_or_init(|| {
        let path = std::env::var("CRAB_FRA_NET").unwrap_or_else(|_| "nets/fra-champion.safetensors".to_string());
        match net_eval::load_slot(net_eval::SLOT_FRA, std::path::Path::new(&path)) {
            Ok(()) => {
                eprintln!("FRA value net loaded from {path}; the local bot searches FRA games on its leaf");
                true
            }
            Err(e) => {
                eprintln!("FRA value net {path}: {e}; FRA games keep the champion leaf");
                false
            }
        }
    })
}

/// Worker threads for the bot's root search.
///
/// The ladder and the training actors run one game per core and leave the
/// search serial; this program runs *one* game and a human waits on every
/// decision of it, so the cores are sitting there. The budget is split
/// rather than multiplied (`MctsConfig::search_threads`), so this is
/// latency, not strength — see that field for the trade and the gate.
///
/// Half the machine, capped at 4. Half because Bevy has its own task and
/// render pools on this box and a search that takes every core stutters the
/// frame the human is looking at; capped at 4 because the budget is 256
/// rollouts and splitting it further leaves each worker's UCB1 allocating on
/// too few samples of its own.
pub(super) fn bot_search_threads() -> usize {
    std::thread::available_parallelism().map(|n| (n.get() / 2).clamp(1, 4)).unwrap_or(1)
}
