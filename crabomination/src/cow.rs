//! Copy-on-write box for snapshot-heavy state.
//!
//! The type lives in `crabomination_base` so [`crate::card::CardData`] can
//! hold its own cold group behind one; this module re-exports it under the
//! path the engine has always used.

pub use crabomination_base::cow::CowBox;

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
    /// has to be inline.
    #[test]
    fn game_state_stays_small() {
        let n = std::mem::size_of::<crate::game::GameState>();
        assert!(n <= 1_536, "GameState grew to {n} bytes (cap 1,536) — see PERF (-144)");
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

