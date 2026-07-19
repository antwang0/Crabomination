//! Functionality tests for the Secrets of Strixhaven card pack
//! (`catalog::sets::sos`). Mirrors `tests/modern.rs`: each card gets at
//! least one test exercising its primary play pattern. Grouped into one
//! integration-test binary to keep link time and `target/` size in check.

use crabomination::card::{CardDefinition, CounterType};
use crabomination::game::{CardId, GameState};

mod colors;
mod batch1;
mod batch2;
mod push3_7;
mod push8_10;
mod push15_17;
mod prismari_ward;
mod mana_shapes;
mod ward_misc;
mod hybrid_lands;

/// Put `def` onto `seat`'s battlefield already prepared (one Prepared
/// counter), returning its id — the standard setup for prepare-spell
/// casts. Bypasses the ETB path, so it works for trigger-prepared
/// cards too.
pub(crate) fn prepared_on_battlefield(
    g: &mut GameState,
    seat: usize,
    def: CardDefinition,
) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == id) {
        c.add_counters(CounterType::Prepared, 1);
    }
    id
}
