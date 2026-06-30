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
    /// Cumulative connections accepted (lifetime). The denominator the
    /// refusal counters lacked: `refused / (accepted + refused)` is the
    /// pressure ratio, so operators can tell "a few refusals against heavy
    /// healthy traffic" from "refusing most of what arrives".
    accepted: u64,
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
    /// Largest number of slots any single IP currently holds. A value
    /// approaching `current` while `distinct_ips` stays low is the clearest
    /// single-source-abuse signal — sharper than `distinct_ips` alone.
    pub(crate) max_per_ip: usize,
    /// Lifetime accepted-connection count — the denominator for
    /// [`refusal_rate_pct`](Self::refusal_rate_pct).
    pub(crate) accepted: u64,
}

impl SlotSnapshot {
    /// Percentage of all connection *attempts* (accepted + refused) that
    /// were turned away. 0 when nothing has arrived yet. A rising rate is
    /// the single clearest "the server is under-provisioned or under
    /// attack" signal — refusal counts alone don't show whether 100
    /// refusals is a blip against heavy traffic or near-total rejection.
    pub(crate) fn refusal_rate_pct(&self) -> u64 {
        let refused = self.refused_global.saturating_add(self.refused_per_ip);
        let attempts = self.accepted.saturating_add(refused);
        refused.saturating_mul(100).checked_div(attempts).unwrap_or(0)
    }
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
        state.accepted += 1;
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
            max_per_ip: state.per_ip.values().copied().max().unwrap_or(0),
            accepted: state.accepted,
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


#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn ip(n: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, n))
    }

    #[test]
    fn global_cap_refuses_when_full() {
        let mgr = SlotManager::new(2, 0);
        let _a = mgr.try_acquire(ip(1)).expect("first slot");
        let _b = mgr.try_acquire(ip(2)).expect("second slot");
        assert!(matches!(mgr.try_acquire(ip(3)), Err(SlotRefusal::GlobalCapReached)));
    }

    #[test]
    fn per_ip_cap_is_independent_per_address() {
        let mgr = SlotManager::new(0, 1);
        let _a = mgr.try_acquire(ip(1)).expect("ip1 first");
        // Same IP is now full…
        assert!(matches!(mgr.try_acquire(ip(1)), Err(SlotRefusal::PerIpCapReached)));
        // …but a different IP still has its own slot.
        let _b = mgr.try_acquire(ip(2)).expect("ip2 first");
    }

    #[test]
    fn dropping_a_guard_frees_its_slot() {
        let mgr = SlotManager::new(1, 0);
        {
            let _a = mgr.try_acquire(ip(1)).expect("slot");
            assert!(matches!(mgr.try_acquire(ip(2)), Err(SlotRefusal::GlobalCapReached)));
        }
        // The guard dropped at the end of the block; the slot is reusable.
        let _b = mgr.try_acquire(ip(2)).expect("slot freed after drop");
    }

    #[test]
    fn per_ip_entry_is_removed_when_count_hits_zero() {
        let mgr = SlotManager::new(0, 1);
        {
            let _a = mgr.try_acquire(ip(1)).expect("slot");
        }
        // After release the IP can acquire again (entry was cleaned up, not stuck at 1).
        let _b = mgr.try_acquire(ip(1)).expect("ip reusable after release");
    }

    #[test]
    fn snapshot_max_per_ip_tracks_busiest_address() {
        let mgr = SlotManager::new(0, 0);
        let _a = mgr.try_acquire(ip(1)).expect("ip1 #1");
        let _b = mgr.try_acquire(ip(1)).expect("ip1 #2");
        let _c = mgr.try_acquire(ip(2)).expect("ip2 #1");
        let s = mgr.snapshot();
        assert_eq!(s.distinct_ips, 2);
        assert_eq!(s.max_per_ip, 2, "ip1 holds the most slots");
        assert_eq!(s.current, 3);
    }

    #[test]
    fn refusal_rate_tracks_accepts_and_refusals() {
        let mgr = SlotManager::new(2, 0);
        let _a = mgr.try_acquire(ip(1)).expect("accept 1");
        let _b = mgr.try_acquire(ip(2)).expect("accept 2");
        // Two more attempts bounce off the full global cap.
        assert!(mgr.try_acquire(ip(3)).is_err());
        assert!(mgr.try_acquire(ip(4)).is_err());
        let s = mgr.snapshot();
        assert_eq!(s.accepted, 2, "two accepted");
        assert_eq!(s.refused_global, 2, "two refused");
        // 2 refused / 4 attempts = 50%.
        assert_eq!(s.refusal_rate_pct(), 50);
    }

    #[test]
    fn refusal_rate_is_zero_with_no_traffic() {
        assert_eq!(SlotManager::new(0, 0).snapshot().refusal_rate_pct(), 0);
    }

    #[test]
    fn zero_caps_mean_unlimited() {
        let mgr = SlotManager::new(0, 0);
        let mut guards = Vec::new();
        for _ in 0..50 {
            guards.push(mgr.try_acquire(ip(1)).expect("unlimited"));
        }
    }
}
