//! CR 104.4b / 732.4 — a mandatory loop that never repeats a board because
//! every cycle adds an object (Gremlin Tamer under Secret Arcade: each Gremlin
//! token is an enchantment, so its arrival re-triggers Eerie) or a counter
//! (Heliod, Sun-Crowned and a lifelinking Shalai and Hallar). The exact-state
//! watch in `stack.rs` cannot see it; this one counts the consecutive
//! triggered-ability resolutions that put a permanent onto the battlefield
//! (a non-growing rider between them is carried, not counted) and never leave
//! the stack shallower than the last growing resolution found it. Each
//! resolution pops one item, so holding the depth for
//! [`GROWTH_LOOP_DRAW_RESOLUTIONS`] resolutions means every one pushed a
//! successor — a loop, not a finite cascade (a cascade from one event pushes
//! its triggers at once and then drains).

use super::{GameEvent, GameState};

/// Far above any finite chain; below the simulators' board bound (1,024
/// permanents), which a one-token-per-cycle loop would otherwise reach first.
pub const GROWTH_LOOP_DRAW_RESOLUTIONS: u32 = 400;

/// A trigger resolving this deep counts as growth on its own: the stack is
/// what the loop grows. Syr Konrad under The Mindskinner (each ping becomes
/// three mills, each milled creature card another ping) beside Vigor,
/// Worldspine Wurm and Kozilek, Butcher of Truth (each shuffles itself or its
/// graveyard back) never repeats a board and never adds a permanent; its
/// stack climbed past 3,000 to the action cap (sweep seed 113, block 11352).
/// The board bound: a finite fan-out this deep pushes its triggers at once
/// and drains below the chain's floor on the next resolution.
pub const STACK_LOOP_DEPTH: usize = crate::recommend::MAX_BATTLEFIELD;

impl GameState {
    /// Called after each top-of-stack resolution with the watch's previous
    /// `(floor, tagged)` — `floor` the chain's first pre-resolution depth (0
    /// idle), `tagged` a 10-bit count over a 6-bit turn/step tag — and
    /// returns the next. A chain is consecutive trigger resolutions in one
    /// step whose pre-resolution depth never drops below its first (the
    /// successor may be pushed after this returns); only those that grew the
    /// battlefield count, a non-growing rider between them is carried. A chain
    /// that only moves life totals is the exact watch's, unless the life lost
    /// is a seat's that is still in at 0 or less (the caller counts that).
    pub(crate) fn note_growth_loop(
        &mut self,
        was_trigger: bool,
        depth_before: usize,
        grew: bool,
        (floor, tagged): (u32, u32),
        events: &mut Vec<GameEvent>,
    ) -> (u32, u32) {
        if !was_trigger || self.game_over.is_some() {
            return (0, 0);
        }
        let tag = (self.turn_number.wrapping_mul(16).wrapping_add(self.step as u32)) & 0x3F;
        let depth = (depth_before as u32).clamp(1, 0xFFFF);
        let (count, chain_tag) = (tagged >> 6, tagged & 0x3F);
        // Depth growth holds the floor where it is: the loop's depth
        // oscillates (each resolution pops one, every few push several).
        let deep_only = !grew && depth_before >= STACK_LOOP_DEPTH;
        let grew = grew || deep_only;
        if floor == 0 || chain_tag != tag || depth < floor {
            return if grew { (depth, tag) } else { (0, 0) };
        }
        // A rider in the cycle that adds nothing (Doomwake Giant's -1/-1
        // beside a Ghostly Dancers token) keeps the chain without counting.
        if !grew {
            return (floor, tagged);
        }
        let count = count + 1;
        if count >= GROWTH_LOOP_DRAW_RESOLUTIONS {
            self.game_over = Some(None);
            events.push(GameEvent::GameOver { winner: None });
            return (0, 0);
        }
        // The floor ratchets to each growing resolution's depth: a loop
        // re-pushes at the same depth (or deeper), while a cascade a single
        // event fanned out drains below it and starts over (a Lumra land
        // drop under 160 Scute Swarms is 400 growing resolutions, all
        // above the depth-1 trigger that began them — seed 4100128).
        (if deep_only { floor } else { floor.max(depth) }, count << 6 | tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(g: &mut GameState, depths: impl Iterator<Item = usize>) -> bool {
        let (mut w, mut events) = ((0, 0), Vec::new());
        for d in depths {
            w = g.note_growth_loop(true, d, true, w, &mut events);
        }
        g.game_over.is_some()
    }

    /// CR 104.4b — a growing trigger that re-pushes at one depth is a loop.
    #[test]
    fn a_constant_depth_growing_chain_draws() {
        let mut g = crate::game::multi_player_game(2);
        assert!(run(&mut g, std::iter::repeat_n(1, GROWTH_LOOP_DRAW_RESOLUTIONS as usize + 1)));
        assert_eq!(g.game_over, Some(None));
    }

    /// CR 104.4b — a chain that grows only the stack past `STACK_LOOP_DEPTH`
    /// (every third resolution pushes four, the others push none) is a loop.
    #[test]
    fn a_climbing_stack_with_no_board_growth_draws() {
        let mut g = crate::game::multi_player_game(2);
        let (mut w, mut events, mut d) = ((0, 0), Vec::new(), STACK_LOOP_DEPTH);
        for i in 0..GROWTH_LOOP_DRAW_RESOLUTIONS as usize * 2 {
            w = g.note_growth_loop(true, d, false, w, &mut events);
            d = if i % 3 == 2 { d + 4 } else { d - 1 };
        }
        assert_eq!(g.game_over, Some(None));
    }

    /// A deep stack that only drains is a finite cascade, not a loop.
    #[test]
    fn a_draining_deep_stack_does_not_draw() {
        let mut g = crate::game::multi_player_game(2);
        let (mut w, mut events) = ((0, 0), Vec::new());
        for d in (STACK_LOOP_DEPTH..STACK_LOOP_DEPTH * 2).rev() {
            w = g.note_growth_loop(true, d, false, w, &mut events);
        }
        assert!(g.game_over.is_none());
    }

    /// CR 104.4b — a finite fan-out that drains (450 Scute Swarm copies off
    /// one land) grows the board 450 times but is no loop.
    #[test]
    fn a_draining_growing_cascade_does_not_draw() {
        let mut g = crate::game::multi_player_game(2);
        assert!(!run(&mut g, std::iter::once(1).chain((1..=450).rev())));
    }
}
