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

/// CR 309.4c — a room ability uses the stack; CR 309.6 — the finished dungeon
/// leaves the game only once that ability has resolved.
#[test]
fn cr_309_6_dungeon_leaves_the_game_after_its_last_room_resolves() {
    let mut g = two_player_game();
    let sword = g.add_card_to_battlefield(0, catalog::shortcut_seeker());
    g.add_card_to_library(0, catalog::grizzly_bears()); // the Temple's draw
    g.players[0].dungeon = Some(("Lost Mine of Phandelver".into(), 5));
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(sword);

    let evs = g.resolve_effect(&crabomination::effect::Effect::Venture, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    assert!(!g.stack.is_empty(), "the room ability went on the stack");
    assert!(g.players[0].dungeon.is_some(), "still in the dungeon while it resolves");

    drain_stack(&mut g);
    assert!(g.players[0].dungeon.is_none(), "the dungeon left the game");
    assert_eq!(g.players[0].dungeons_completed, 1);
}
