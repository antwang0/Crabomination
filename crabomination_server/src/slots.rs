//! Concurrent-connection accounting: global and per-IP caps enforced via
//! RAII slot guards.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

/// Tracks concurrent connections to enforce global and per-IP caps.
///
/// One [`SlotGuard`] is acquired per accepted connection (so a pair-mode
/// match holds two slots — one per seat, each indexed by the seat's own
/// peer IP). The guard's `Drop` impl releases the counters, so a panicking
/// match thread still frees its slot.
///
/// Per-IP limits operate on the raw remote address, so clients behind a
/// shared NAT or load balancer share one counter. That's the right behavior
/// for a hobby server (the only signal we have is the socket-level peer
/// address); production setups would want X-Forwarded-For unwrapping at a
/// reverse-proxy layer above us.
#[derive(Clone)]
pub(crate) struct SlotManager {
    inner: Arc<Mutex<SlotState>>,
    /// 0 = unlimited.
    pub(crate) global_cap: usize,
    /// 0 = unlimited.
    pub(crate) per_ip_cap: usize,
}

#[derive(Default)]
pub(crate) struct SlotState {
    total: usize,
    per_ip: HashMap<IpAddr, usize>,
    /// High-water mark of `total` — the most concurrent connections ever
    /// held at once. Lets operators size the global cap against real peak
    /// load rather than guessing.
    peak: usize,
    /// Cumulative connections refused because the global cap was full.
    refused_global: u64,
    /// Cumulative connections refused because the per-IP cap was full —
    /// distinct from `refused_global` so a single abusive IP hammering the
    /// per-IP cap reads differently from genuine global saturation.
    refused_per_ip: u64,
}

/// Point-in-time view of the slot accounting, for operator telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SlotSnapshot {
    pub(crate) current: usize,
    pub(crate) peak: usize,
    pub(crate) refused_global: u64,
    pub(crate) refused_per_ip: u64,
    /// Distinct remote IPs currently holding at least one slot. Read against
    /// `current` it separates "one IP holding many slots" (possible abuse)
    /// from "many IPs each holding a few" (healthy load).
    pub(crate) distinct_ips: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SlotRefusal {
    GlobalCapReached,
    PerIpCapReached,
}

impl SlotManager {
    pub(crate) fn new(global_cap: usize, per_ip_cap: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SlotState::default())),
            global_cap,
            per_ip_cap,
        }
    }

    pub(crate) fn try_acquire(&self, addr: IpAddr) -> Result<SlotGuard, SlotRefusal> {
        // Poisoning here means a previous holder panicked while updating
        // counters. The state is still structurally valid (we only do
        // small arithmetic under the lock), so recover via `into_inner`
        // instead of propagating the panic.
        let mut state = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if self.global_cap != 0 && state.total >= self.global_cap {
            state.refused_global += 1;
            return Err(SlotRefusal::GlobalCapReached);
        }
        if self.per_ip_cap != 0 {
            let count = state.per_ip.get(&addr).copied().unwrap_or(0);
            if count >= self.per_ip_cap {
                state.refused_per_ip += 1;
                return Err(SlotRefusal::PerIpCapReached);
            }
        }
        state.total += 1;
        state.peak = state.peak.max(state.total);
        *state.per_ip.entry(addr).or_insert(0) += 1;
        Ok(SlotGuard {
            inner: Arc::clone(&self.inner),
            addr,
        })
    }

    /// Snapshot the live occupancy + cumulative refusal counters for a
    /// rolling telemetry line. Recovers a poisoned lock (see `try_acquire`).
    pub(crate) fn snapshot(&self) -> SlotSnapshot {
        let state = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        SlotSnapshot {
            current: state.total,
            peak: state.peak,
            refused_global: state.refused_global,
            refused_per_ip: state.refused_per_ip,
            distinct_ips: state.per_ip.len(),
        }
    }
}

/// RAII handle that releases a slot when dropped.
pub(crate) struct SlotGuard {
    inner: Arc<Mutex<SlotState>>,
    addr: IpAddr,
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        let mut state = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        state.total = state.total.saturating_sub(1);
        if let Some(c) = state.per_ip.get_mut(&self.addr) {
            *c = c.saturating_sub(1);
            if *c == 0 {
                state.per_ip.remove(&self.addr);
            }
        }
    }
}

