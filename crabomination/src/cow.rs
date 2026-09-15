//! Copy-on-write box for snapshot-heavy state.
//!
//! The type lives in `crabomination_base` so [`crate::card::CardData`] can
//! hold its own cold group behind one; this module re-exports it under the
//! path the engine has always used.

pub use crabomination_base::cow::{CowBox, make_mut, unique_mut};

#[cfg(test)]
mod tests {
    use super::*;

    /// The `perform_action` transaction contract: a rejected action
    /// restores the exact pre-action state (TODO "Rollback / Undo"
    /// Phase 1). Serialized-state equality is the strongest observable
    /// check without a `PartialEq` on `GameState`.
    #[test]
    fn rejected_action_restores_state_exactly() {
        use crate::game::{GameAction, GameState};
        use crate::player::Player;
        let mut g = GameState::new(vec![Player::new(0, "A"), Player::new(1, "B")]);
        let id = g.add_card_to_hand(0, crate::catalog::shivan_dragon());
        g.priority.player_with_priority = 0;
        let before = serde_json::to_string(&g).unwrap();
        // No mana in pool and no lands: the cast is rejected during payment.
        let r = g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        });
        assert!(r.is_err(), "unpayable cast is rejected");
        assert_eq!(serde_json::to_string(&g).unwrap(), before, "rejected action left no trace");
    }

    /// `GameState`'s size is the whole cost of a probe clone once the
    /// containers are behind CoW handles — 544 Ir a clone is a `memcpy`, not
    /// calls (PERF `(-143)`/`(-144)`). Two fields were 53 % of it and `None`
    /// on essentially every clone; boxing them halved the struct. This guard
    /// is what stops the next 800-byte field from landing inline unnoticed.
    /// Raise it only with a `--bench`/callgrind reading that says the field
    /// has to be inline. Raised 1,536 -> 1,600 at PERF `(-280)`: the two
    /// hot-written scratch fields (64 bytes) left the CoW group, sealed
    /// -0.59 % / cube -0.60 % / fixed -0.89 % Ir. Raised 1,600 -> 1,616 at
    /// PERF `(-306)`: `Battlefield`'s write counter and the packed board fold
    /// it stamps (16 bytes, inline because the memo has to travel with the
    /// zone through a clone), fixed -0.208 / cube -0.240 / sealed -0.383 %
    /// Ir *with* the growth already in the reading. Raised 1,616 -> 1,664 at
    /// PERF `(-311)`: the state-level gather memo's key, its `Arc` and its
    /// `SecondPass`, carried inline so the memo travels with the state through
    /// a clone — fixed -0.911 / cube -1.490 / sealed -0.517 % Ir *with* the
    /// growth already in the reading. ⚠ The key's five scalars are stored and
    /// **compared**, not hashed, and that is 16 of the 48 bytes: a 64-bit fold
    /// of small counters is a probabilistic witness and the first cut's fold
    /// collided inside one sweep cell.
    /// Raised 1,664 -> 1,672 at PERF `(-318)`: `Battlefield`'s attachment
    /// fold, one `AtomicU32` (a 31-bit `writes` stamp and its one-bit answer)
    /// that costs eight bytes because the zone is `u64`-aligned and carries no
    /// hole — fixed -0.061 / cube -0.418 / sealed -0.195 % Ir **with the
    /// growth already in the reading**, since the A/B clones the bigger state
    /// on every probe. ⚠ The free alternative was stealing bits from
    /// `sba_fold`'s word (its legendary count needs 11 of the 28 it has) under
    /// a shared stamp with a per-half "computed" flag; it is sound and it is
    /// written down here rather than taken, because it re-plumbs the SBA
    /// memo's store path for eight bytes.
    #[test]
    fn game_state_stays_small() {
        let n = std::mem::size_of::<crate::game::GameState>();
        assert!(n <= 1_672, "GameState grew to {n} bytes (cap 1,672) — see PERF (-144), (-280), (-318)");
    }

    #[test]
    fn iteration_and_serde_round_trip() {
        let zone: CowBox<Vec<u32>> = vec![5, 6].into();
        let doubled: Vec<u32> = (&zone).into_iter().map(|x| x * 2).collect();
        assert_eq!(doubled, vec![10, 12]);
        let json = serde_json::to_string(&zone).unwrap();
        assert_eq!(json, "[5,6]");
        let back: CowBox<Vec<u32>> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, zone);
    }
}

