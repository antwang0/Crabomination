//! AIVAT-style luck correction for the ladder (Burch, Schmid, Moravčík,
//! Morrill & Bowling, "AIVAT: A New Variance Reduction Technique for Agent
//! Evaluation in Imperfect Information Games", AAAI 2018), on the chance
//! events a sealed game has: cards coming off the top of a library.
//!
//! For each such event — the state `s` just before seat `p` takes its top
//! card — the luck of the draw is
//!
//! ```text
//!     luck = V_p(s · actual card) − Σ_c  n_c / N · V_p(s · c)
//! ```
//!
//! over the distinct cards `c` in the library (`n_c` copies of `N`), `V_p`
//! the value net's win probability for `p` from `p`'s own view. Its
//! expectation over a uniformly random top card is exactly zero whatever
//! `V` is, so subtracting the game's summed luck from its outcome leaves the
//! mean unchanged and removes the part of the outcome a good value net can
//! attribute to the deal. A worse `V` costs variance, never bias.
//!
//! ⚠ **The one assumption: the top card is uniform over the library.** That
//! holds after a shuffle and fails after anything that arranged or revealed
//! the top (scry, surveil, "put on top", a revealed top): the engine keeps no
//! record of library knowledge, so those draws are corrected as if random.
//! The bias that leaves is the *skill* of the arranging (a good scry reads
//! as luck and is subtracted), small and the same for both pilots of a
//! mirror; the ladder prints the raw estimate beside the corrected one so a
//! disagreement shows.

use crate::game::GameState;

/// The luck of one logged draw for the drawing seat `p`, by the value net
/// in `slot`; `None` when the slot has no net or the library is empty.
pub fn draw_luck(pre: &GameState, p: usize, slot: u8) -> Option<f64> {
    let lib = &pre.players[p].library;
    let n = lib.len();
    if n == 0 {
        return None;
    }
    // Distinct cards by name, with copies and one library index each.
    let mut groups: Vec<(&'static str, usize, usize)> = Vec::new();
    for (i, c) in lib.iter().enumerate() {
        match groups.iter_mut().find(|g| g.0 == c.definition.name) {
            Some(g) => g.1 += 1,
            None => groups.push((c.definition.name, 1, i)),
        }
    }
    let actual = lib[0].definition.name;
    let (mut expect, mut got) = (0.0f64, None);
    for (name, copies, at) in groups {
        let mut s = pre.clone();
        let lib = &mut s.players[p].library;
        let card = lib.remove(at);
        lib.insert(0, card);
        s.players[p].draw_top();
        let v = f64::from(super::net_eval::win_prob(&s, p, slot)?);
        expect += copies as f64 / n as f64 * v;
        if name == actual {
            got = Some(v);
        }
    }
    Some(got? - expect)
}

/// Seat 0's summed luck over a batch of logged draws (a draw by seat 1
/// counts against seat 0), by the value net in `slot`. Events the net
/// cannot score add nothing.
pub fn seat0_luck(events: &[(GameState, usize)], slot: u8) -> f64 {
    events
        .iter()
        .filter_map(|(s, p)| draw_luck(s, *p, slot).map(|l| if *p == 0 { l } else { -l }))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// With no net in the slot there is nothing to score — the estimator
    /// degrades to the raw outcome rather than inventing a correction.
    #[test]
    fn an_empty_slot_scores_no_luck() {
        let mut g = crate::game::two_player_game();
        g.add_card_to_library(0, crate::catalog::forest());
        assert_eq!(draw_luck(&g, 0, 250), None);
        assert_eq!(seat0_luck(&[(g, 0)], 250), 0.0);
    }

    /// The draw log records the real game's draws — the state before each,
    /// with the drawing seat — only while it is on.
    #[test]
    fn the_draw_log_records_only_while_on() {
        let mut g = crate::game::two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, crate::catalog::forest());
        }
        let mut ev = Vec::new();
        g.draw_one(0, &mut ev);
        assert!(crate::game::draw_log_take().is_empty(), "off by default");
        crate::game::draw_log_start();
        g.draw_one(0, &mut ev);
        let log = crate::game::draw_log_take();
        assert_eq!(log.len(), 1);
        assert_eq!((log[0].0.players[0].library.len(), log[0].1), (2, 0), "the state before the draw");
        g.draw_one(0, &mut ev);
        assert!(crate::game::draw_log_take().is_empty(), "off again after the take");
    }
}
