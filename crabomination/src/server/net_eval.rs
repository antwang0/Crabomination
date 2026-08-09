//! Global registry of loaded value nets, keyed by the small slot id that
//! [`EvalWeights::net_slot`](super::bot::EvalWeights::net_slot) carries.
//!
//! Why a registry and not a field: `EvalWeights` is `Copy` and threads
//! through ~30 free evaluation functions (`eval_material`, the combat
//! sims, `settle_answer`, ...). Handing each of them an `Arc<PlayNet>`
//! would mean rewriting every signature in the bot's hot path; a one-byte
//! slot id keeps the profile system intact and lets two nets coexist in
//! one process — which the training gatekeeper needs (candidate vs best).
//!
//! The hot path pays one atomic load per lookup: each thread caches the
//! slot table and refreshes only when the write generation moves, so
//! evaluation inside simulation loops never touches the `RwLock`.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

use crabomination_nn::{NetEvaluator, PlayNet};

use super::encode::{Vocab, encode_state};
use crate::game::GameState;

/// The promoted net actors and ladder profiles play with.
pub const SLOT_BEST: u8 = 1;
/// The training loop's un-promoted checkpoint, for gatekeeper matches.
pub const SLOT_CANDIDATE: u8 = 2;
const NUM_SLOTS: usize = 3;

/// The slot table: one optional evaluator per slot, shared by the global
/// `SLOTS` and each thread's generation-tagged `CACHE`.
type SlotTable = [Option<Arc<dyn NetEvaluator>>; NUM_SLOTS];

static SLOTS: RwLock<SlotTable> =
    RwLock::new([None, None, None]);
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// The encoder vocabulary every net in the registry must match. Built
/// once; the ML scope is SOS sealed (see `encode`).
pub fn vocab() -> &'static Vocab {
    static VOCAB: OnceLock<Vocab> = OnceLock::new();
    VOCAB.get_or_init(Vocab::sos_sealed)
}

/// Install (or clear, with `None`) a slot — a local [`PlayNet`] or any
/// other [`NetEvaluator`] (the training harness installs a batched GPU
/// client here). Running bots pick the change up at their next
/// evaluation via the generation counter.
pub fn set_slot(slot: u8, net: Option<Arc<dyn NetEvaluator>>) {
    assert!((slot as usize) < NUM_SLOTS && slot != 0, "slot {slot} out of range");
    SLOTS.write().unwrap()[slot as usize] = net;
    GENERATION.fetch_add(1, Ordering::Release);
}

/// Read a safetensors weights file into a slot, checking it against the
/// encoder vocabulary first — a size mismatch means the net was trained
/// against a different card list and its indices would silently mean the
/// wrong cards.
pub fn load_slot(slot: u8, path: &std::path::Path) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let net = PlayNet::load(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    if net.vocab_size() != vocab().size() {
        return Err(format!(
            "{}: net vocab {} != encoder vocab {}",
            path.display(),
            net.vocab_size(),
            vocab().size()
        ));
    }
    set_slot(slot, Some(Arc::new(net)));
    Ok(())
}

thread_local! {
    static CACHE: RefCell<(u64, SlotTable)> =
        const { RefCell::new((u64::MAX, [None, None, None])) };
}

fn net_for(slot: u8) -> Option<Arc<dyn NetEvaluator>> {
    let generation = GENERATION.load(Ordering::Acquire);
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.0 != generation {
            *c = (generation, SLOTS.read().unwrap().clone());
        }
        c.1.get(slot as usize)?.clone()
    })
}

/// The net's win probability for `seat`, or `None` when the slot is empty
/// (callers fall back to the heuristic).
pub fn win_prob(state: &GameState, seat: usize, slot: u8) -> Option<f32> {
    let net = net_for(slot)?;
    Some(net.eval(encode_state(state, seat, vocab())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination_nn::{GLOBAL_FEATS, NUM_GROUPS, OBJ_FEATS, to_safetensors};

    /// A minimal valid net whose output depends only on the bias, so the
    /// probability is predictable without arithmetic.
    fn constant_net(logit: f32) -> PlayNet {
        let h = 1usize;
        let trunk_in = NUM_GROUPS * 2 * h + GLOBAL_FEATS;
        let bytes = to_safetensors(&[
            ("emb.weight", vec![vocab().size(), 1], vec![0.0; vocab().size()]),
            ("obj.weight", vec![1, 1 + OBJ_FEATS], vec![0.0; 1 + OBJ_FEATS]),
            ("obj.bias", vec![1], vec![0.0]),
            ("trunk1.weight", vec![1, trunk_in], vec![0.0; trunk_in]),
            ("trunk1.bias", vec![1], vec![0.0]),
            ("trunk2.weight", vec![1, 1], vec![0.0]),
            ("trunk2.bias", vec![1], vec![0.0]),
            ("head_win.weight", vec![1, 1], vec![0.0]),
            ("head_win.bias", vec![1], vec![logit]),
        ]);
        PlayNet::load(&bytes).expect("constant net loads")
    }

    #[test]
    fn slots_swap_and_thread_cache_follows() {
        let g = GameState::new(vec![
            crate::player::Player::new(0, "A"),
            crate::player::Player::new(1, "B"),
        ]);
        // Empty slot: no answer, callers keep the heuristic.
        assert!(win_prob(&g, 0, SLOT_CANDIDATE).is_none());

        set_slot(SLOT_CANDIDATE, Some(Arc::new(constant_net(0.0))));
        let p = win_prob(&g, 0, SLOT_CANDIDATE).expect("net answers");
        assert!((p - 0.5).abs() < 1e-6, "zero logit is a coin flip, got {p}");

        // Swapping the slot is visible without thread churn.
        set_slot(SLOT_CANDIDATE, Some(Arc::new(constant_net(2.0))));
        let p2 = win_prob(&g, 0, SLOT_CANDIDATE).expect("swapped net answers");
        assert!(p2 > 0.85, "logit 2 ≈ 0.88, got {p2}");

        set_slot(SLOT_CANDIDATE, None);
        assert!(win_prob(&g, 0, SLOT_CANDIDATE).is_none());
    }
}
