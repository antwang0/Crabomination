//! Pre-built demo decks shared between the singleplayer client setup and the
//! TCP server binary. The server skips the mulligan phase, so this builder
//! also deals the standard 7-card opening hands.
//!
//! P0 plays the **BRG combo** deck (Cosmogoyf + Thud, Pact-style). P1 plays
//! the **Goryo's Vengeance** reanimator deck. Many of the cards in these
//! decks are stubs — see `DECK_FEATURES.md` at the repo root for the
//! per-card and per-engine-feature progress tracker.

use rand::seq::SliceRandom;

use crate::card::CardDefinition;
use crate::catalog::*;
use crate::game::GameState;
use crate::player::Player;

type CardFactory = fn() -> CardDefinition;

/// Build a fresh demo match: two seats, shuffled 60-card decks, 7 cards drawn
/// into each opening hand. `wants_ui` is set on both seats so every decision
/// surfaces as a `pending_decision` for the human/bot to answer.
pub fn build_demo_state() -> GameState {
    use rand::RngExt;
    build_demo_state_seeded(rand::rng().random())
}

/// [`build_demo_state`] with the deal pinned to `seed`, drawn from the
/// state's own stream (`GameState.rng`) rather than the thread RNG — which is
/// what `game::rng`'s doc has asked of every shuffle since it was written.
/// Pair it with `server::bot::set_jitter_seed` on the thread that plays the
/// match and a bot-vs-bot run replays exactly.
pub fn build_demo_state_seeded(seed: u64) -> GameState {
    let mut state = GameState::new(vec![
        Player::new(0, "Player 0"),
        Player::new(1, "Player 1"),
    ]);
    state.rng.reseed(seed);

    let p0_deck: &[CardFactory] = brg_combo_deck();
    let p1_deck: &[CardFactory] = goryos_vengeance_deck();

    for &f in p0_deck { state.add_card_to_library(0, crate::cube::card_arc(f)); }
    state.players[0].library.shuffle(&mut state.rng.draw());
    for &f in p1_deck { state.add_card_to_library(1, crate::cube::card_arc(f)); }
    state.players[1].library.shuffle(&mut state.rng.draw());

    state.players[0].wants_ui = true;
    state.players[1].wants_ui = true;

    state
}

/// 60-card BRG combo deck (Cosmogoyf + Thud + Pact). Player 0's deck.
pub fn brg_combo_deck() -> &'static [CardFactory] {
    &[
        // Lands (25)
        blackcleave_cliffs, blackcleave_cliffs, blackcleave_cliffs, blackcleave_cliffs,
        blightstep_pathway, blightstep_pathway,
        blooming_marsh, blooming_marsh, blooming_marsh, blooming_marsh,
        copperline_gorge, copperline_gorge, copperline_gorge, copperline_gorge,
        darkbore_pathway, darkbore_pathway,
        gemstone_caverns, gemstone_caverns, gemstone_caverns, gemstone_caverns,
        gemstone_mine, gemstone_mine, gemstone_mine, gemstone_mine,
        swamp,
        // Creatures (13)
        callous_sell_sword,
        chancellor_of_the_tangle, chancellor_of_the_tangle, chancellor_of_the_tangle, chancellor_of_the_tangle,
        // Tarmogoyf (graveyard-scaled) — the BRG deck wants the
        // all-graveyards CDA, so the stand-in points at the real Tarmogoyf
        // (Cosmogoyf is now the faithful EOE exile-scaled card).
        tarmogoyf, tarmogoyf, tarmogoyf, tarmogoyf,
        devourer_of_destiny, devourer_of_destiny, devourer_of_destiny, devourer_of_destiny,
        // Spells (22)
        pact_of_negation, pact_of_negation, pact_of_negation, pact_of_negation,
        plunge_into_darkness, plunge_into_darkness, plunge_into_darkness, plunge_into_darkness,
        serum_powder, serum_powder, serum_powder, serum_powder,
        spoils_of_the_vault, spoils_of_the_vault, spoils_of_the_vault, spoils_of_the_vault,
        summoners_pact, summoners_pact,
        thud, thud, thud, thud,
    ]
}

// ── Commander demo ─────────────────────────────────────────────────────────

/// Build a 4-player Commander free-for-all over the four
/// [`crate::pod::target_decks`] — one deck per seat, so the demo is the same
/// pod the smoke test plays. `apply_format` sets 40 life and the 100-card /
/// singleton rules; `seat_commanders` puts each commander in its command zone
/// with the CR 903.9b replacement registered.
pub fn build_commander_state() -> GameState {
    use rand::RngExt;
    build_commander_state_seeded(rand::rng().random())
}

/// [`build_commander_state`] with the deal pinned — see
/// [`build_demo_state_seeded`] for why the shuffle moved onto the state's own
/// stream.
pub fn build_commander_state_seeded(seed: u64) -> GameState {
    let decks = crate::pod::pod_field(4);
    let mut state = crate::pod::build_pod_template(&decks);
    state.rng.reseed(seed);
    for seat in 0..state.players.len() {
        state.players[seat].name = format!("Player {seat}");
        state.players[seat].library.shuffle(&mut state.rng.draw());
    }
    state
}

/// 60-card Goryo's Vengeance reanimator deck. Player 1's deck.
pub fn goryos_vengeance_deck() -> &'static [CardFactory] {
    &[
        // Lands (24)
        cephalid_coliseum,
        flooded_strand, flooded_strand, flooded_strand,
        godless_shrine,
        hallowed_fountain,
        island,
        marsh_flats, marsh_flats, marsh_flats,
        meticulous_archive,
        overgrown_tomb,
        plains,
        polluted_delta, polluted_delta, polluted_delta, polluted_delta,
        shadowy_backstreet,
        swamp,
        undercity_sewers,
        watery_grave,
        // Creatures (17)
        atraxa_grand_unifier, atraxa_grand_unifier, atraxa_grand_unifier, atraxa_grand_unifier,
        griselbrand,
        psychic_frog, psychic_frog, psychic_frog, psychic_frog,
        quantum_riddler, quantum_riddler, quantum_riddler, quantum_riddler,
        solitude, solitude, solitude, solitude,
        // Spells (19)
        ephemerate, ephemerate, ephemerate, ephemerate,
        faithful_mending, faithful_mending, faithful_mending, faithful_mending,
        force_of_negation, force_of_negation, force_of_negation,
        goryos_vengeance, goryos_vengeance, goryos_vengeance, goryos_vengeance,
        prismatic_ending, prismatic_ending, prismatic_ending, prismatic_ending,
        thoughtseize, thoughtseize, thoughtseize,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The seeded builders are the deal half of a reproducible bot match —
    /// `server::bot::set_jitter_seed` is the other. Same seed, same order;
    /// a different seed is a different order, so the seed is actually read.
    #[test]
    fn seeded_demo_builders_reproduce_the_deal() {
        let order = |s: &GameState, seat: usize| -> Vec<u32> {
            s.players[seat].library.iter().map(|c| c.id.0).collect()
        };
        for seat in 0..2 {
            assert_eq!(
                order(&build_demo_state_seeded(7), seat),
                order(&build_demo_state_seeded(7), seat),
            );
        }
        assert_ne!(
            order(&build_demo_state_seeded(7), 0),
            order(&build_demo_state_seeded(8), 0),
        );
        for seat in 0..4 {
            assert_eq!(
                order(&build_commander_state_seeded(7), seat),
                order(&build_commander_state_seeded(7), seat),
            );
        }
        assert_ne!(
            order(&build_commander_state_seeded(7), 0),
            order(&build_commander_state_seeded(8), 0),
        );
    }
}
