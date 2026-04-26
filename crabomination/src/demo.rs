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
    let mut state = GameState::new(vec![
        Player::new(0, "Player 0"),
        Player::new(1, "Player 1"),
    ]);

    let p0_deck: &[CardFactory] = brg_combo_deck();
    let p1_deck: &[CardFactory] = goryos_vengeance_deck();

    let mut rng = rand::rng();
    for &f in p0_deck { state.add_card_to_library(0, f()); }
    state.players[0].library.shuffle(&mut rng);
    for &f in p1_deck { state.add_card_to_library(1, f()); }
    state.players[1].library.shuffle(&mut rng);

    state.players[0].wants_ui = true;
    state.players[1].wants_ui = true;

    state
}

/// 60-card BRG combo deck (Cosmogoyf + Thud + Pact). Player 0's deck.
fn brg_combo_deck() -> &'static [CardFactory] {
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
        cosmogoyf, cosmogoyf, cosmogoyf, cosmogoyf,
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

/// 60-card Goryo's Vengeance reanimator deck. Player 1's deck.
fn goryos_vengeance_deck() -> &'static [CardFactory] {
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
