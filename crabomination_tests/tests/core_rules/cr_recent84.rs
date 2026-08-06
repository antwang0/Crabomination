//! CR conformance for this run:
//! - CR 712 — double-faced cards: a face swap into an instant/sorcery face
//!   does nothing, and a spell told to enter transformed onto such a face
//!   falls through to the graveyard.
//! - CR 701.28 — an "as this transforms" effect resolves inside the flip.

use crabomination::catalog;
use crabomination::game::*;

/// CR 712.10 — the Saga back face of a sorcery DFC can't turn back over.
#[test]
fn cr_712_10_transform_into_a_sorcery_face_does_nothing() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::esper_origins());
    let mut evs = vec![];
    g.transform_permanent(id, &mut evs);
    let saga = g.battlefield_find(id).expect("still on the battlefield");
    assert_eq!(saga.definition.name, "Summon: Esper Maduin", "flipped to the Saga face");

    // Flipping back would land on the Sorcery front face — nothing happens.
    let mut evs = vec![];
    g.transform_permanent(id, &mut evs);
    let still = g.battlefield_find(id).expect("still on the battlefield");
    assert_eq!(still.definition.name, "Summon: Esper Maduin");
    assert!(still.transformed, "the face swap was refused, not half-applied");
    assert!(evs.is_empty(), "no Transformed event for a refused flip");
}

/// CR 701.28 — Sephiroth's Super Nova emblem lands inside the transform, not
/// as a separate trigger afterwards.
#[test]
fn cr_701_28_as_transforms_effect_runs_inside_the_flip() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sephiroth_fabled_soldier());
    assert!(g.players[0].emblems.is_empty());
    let mut evs = vec![];
    g.transform_permanent(id, &mut evs);
    assert_eq!(g.players[0].emblems.len(), 1, "emblem minted during the flip");
    assert!(evs.iter().any(|e| matches!(e, GameEvent::Transformed { .. })));
}
