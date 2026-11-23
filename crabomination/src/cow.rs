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

    /// Behind CoW handles, `GameState`'s size is the whole cost of a probe
    /// clone (a `memcpy`; PERF `(-143)`/`(-144)` boxed the two big
    /// mostly-`None` fields). This guard stops the next large field landing
    /// inline unnoticed; raise it only with a `--bench`/callgrind A/B that
    /// clones the bigger state and still wins. Past raises, each so paid for:
    /// `(-280)`, `(-306)`, `(-311)`, `(-318)`, `(-344)`.
    ///
    /// ⚠ `(-311)`'s gather-memo key stores and **compares** its five scalars
    /// rather than hashing them: a 64-bit fold of small counters collided
    /// inside one sweep cell.
    /// ⚠ `(-318)`'s eight bytes could instead be stolen from `sba_fold`'s
    /// word under a shared stamp — sound, but not taken, since it re-plumbs
    /// the SBA memo's store path for eight bytes.
    #[test]
    fn game_state_stays_small() {
        let n = std::mem::size_of::<crate::game::GameState>();
        assert!(n <= 1_680, "GameState grew to {n} bytes (cap 1,680) — see PERF (-144), (-280), (-318), (-344)");
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

