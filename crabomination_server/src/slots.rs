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
            return Err(SlotRefusal::GlobalCapReached);
        }
        if self.per_ip_cap != 0 {
            let count = state.per_ip.get(&addr).copied().unwrap_or(0);
            if count >= self.per_ip_cap {
                return Err(SlotRefusal::PerIpCapReached);
            }
        }
        state.total += 1;
        *state.per_ip.entry(addr).or_insert(0) += 1;
        Ok(SlotGuard {
            inner: Arc::clone(&self.inner),
            addr,
        })
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

