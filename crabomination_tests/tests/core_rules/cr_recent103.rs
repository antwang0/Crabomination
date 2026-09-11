//! CR 608.2b is checked once, as a spell *begins* to resolve.
//!
//! `continue_spell_resolution` re-ran the target-legality fizzle on every
//! resume pass, and a resume pass is the same resolution continuing after a
//! player choice. So a spell that killed its own target and then asked a
//! question stopped mid-effect: Lash Out deals 3 to a creature, clashes, and
//! suspends for the clash question; by the time the answer arrives the
//! creature is in the graveyard, the re-entry fizzled the spell, and the clash
//! never happened. Only `wants_ui` seats reach it — every bot seat is one —
//! and the abandoned resolution left its answer log behind, which is how
//! `CRAB_ANSWER_LOG=strict` named the arm.

use crabomination::catalog;
use crabomination::game::*;

/// Seat 0 is `wants_ui`, so the clash question suspends; the target dies to
/// Lash Out's own first half before the answer comes back. The clash must
/// still happen.
#[test]
fn a_resumed_spell_does_not_fizzle_on_the_target_it_killed_itself() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    g.add_card_to_library(0, catalog::serra_angel()); // MV 5 reveal — wins
    g.add_card_to_library(1, catalog::lightning_bolt()); // MV 1 reveal
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let lo = g.add_card_to_hand(0, catalog::lash_out());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, lo, Target::Permanent(bear));
    assert!(g.battlefield_find(bear).is_none(), "3 damage killed the bear");

    // The clash question is pending on the `wants_ui` seat, not swallowed.
    let pd = g.pending_decision.as_ref().expect("clash asks seat 0");
    assert!(
        matches!(pd.decision, Decision::OptionalTrigger { .. }),
        "the clash's keep-or-bottom question",
    );
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(false)))
        .expect("keep the revealed card on top");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 17, "clash won: 3 to the dead bear's controller");
    assert!(g.pending_decision.is_none(), "and the resolution finished");
}
