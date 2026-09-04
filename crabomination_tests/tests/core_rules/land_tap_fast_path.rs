//! PERF `(-204)` — the printed land-tap fast path in
//! `activate_ability_inner` must leave the game exactly where the generic
//! path leaves it, on every board it accepts and every board it declines.
//! Each scenario builds the same board twice, taps once down each path
//! (`FORCE_GENERIC_ACTIVATION` is the switch) and compares the returned
//! events and the whole serialized state. The debug-only tally
//! `PLAIN_LAND_TAPS` says which path an activation took, so an "accept"
//! scenario cannot pass vacuously by declining.

use crabomination::catalog;
use crabomination::game::actions::{FORCE_GENERIC_ACTIVATION, PLAIN_LAND_TAPS};
use crabomination::game::*;
use crabomination::mana::Color;
use std::sync::atomic::Ordering::Relaxed;

/// Board builder: the state, the land to tap and the ability index.
type Setup = fn() -> (GameState, CardId, usize);

fn tap(g: &mut GameState, id: CardId, idx: usize) -> String {
    let r = g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    format!("{r:?}")
}

/// Tap once down each path; `accept` says whether the fast path must have
/// taken the activation.
fn both_ways(name: &str, accept: bool, setup: Setup) {
    let (mut fast, id, idx) = setup();
    let (mut slow, id2, idx2) = setup();
    assert_eq!((id, idx), (id2, idx2), "{name}: setup is not deterministic");
    FORCE_GENERIC_ACTIVATION.store(false, Relaxed);
    let before = PLAIN_LAND_TAPS.load(Relaxed);
    let fast_events = tap(&mut fast, id, idx);
    let taken = PLAIN_LAND_TAPS.load(Relaxed) - before;
    FORCE_GENERIC_ACTIVATION.store(true, Relaxed);
    let slow_events = tap(&mut slow, id, idx);
    FORCE_GENERIC_ACTIVATION.store(false, Relaxed);
    assert_eq!(taken, u64::from(accept), "{name}: fast path taken {taken} times");
    assert_eq!(fast_events, slow_events, "{name}: events differ");
    assert_eq!(
        serde_json::to_string(&fast).unwrap(),
        serde_json::to_string(&slow).unwrap(),
        "{name}: state differs"
    );
    if accept {
        assert!(fast_events.starts_with("Ok"), "{name}: {fast_events}");
    }
}

fn land(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.battlefield_find_mut(id).unwrap().tapped = false;
    id
}

#[test]
fn basic_forest() {
    both_ways("forest", true, || {
        let mut g = two_player_game();
        let id = land(&mut g, 0, catalog::forest());
        (g, id, 0)
    });
}

#[test]
fn colorless_wastes() {
    both_ways("wastes", true, || {
        let mut g = two_player_game();
        let id = land(&mut g, 0, catalog::wastes());
        (g, id, 0)
    });
}

#[test]
fn dual_both_indices() {
    both_ways("temple idx 0", true, || {
        let mut g = two_player_game();
        let id = land(&mut g, 0, catalog::temple_of_epiphany());
        (g, id, 0)
    });
    both_ways("temple idx 1", true, || {
        let mut g = two_player_game();
        let id = land(&mut g, 0, catalog::temple_of_epiphany());
        (g, id, 1)
    });
}

/// Mutavault's `{T}: Add {C}` is plain; its animation is not.
#[test]
fn mutavault_mana_yes_animate_no() {
    both_ways("mutavault idx 0", true, || {
        let mut g = two_player_game();
        let id = land(&mut g, 0, catalog::mutavault());
        (g, id, 0)
    });
    both_ways("mutavault idx 1", false, || {
        let mut g = two_player_game();
        let id = land(&mut g, 0, catalog::mutavault());
        g.players[0].mana_pool.add_colorless(1);
        (g, id, 1)
    });
}

/// An animated land is a creature (CR 106.12, CR 602.5g): generic path.
#[test]
fn animated_land_declines() {
    both_ways("animated mutavault", false, || {
        let mut g = two_player_game();
        let id = land(&mut g, 0, catalog::mutavault());
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("animate");
        drain_stack(&mut g);
        (g, id, 0)
    });
}

/// Contamination is a resolver-side replacement, not an activation gate:
/// the fast path accepts and the tap makes {B} either way.
#[test]
fn contamination_accepted() {
    both_ways("contamination", true, || {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::contamination());
        let id = land(&mut g, 0, catalog::forest());
        (g, id, 0)
    });
    let (mut g, id, _) = {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::contamination());
        let id = land(&mut g, 0, catalog::forest());
        (g, id, 0)
    };
    tap(&mut g, id, 0);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
}

/// Bubbling Muck's turn-scoped grant lives beside the mana-static lane; the
/// fast path resolves it through the same CR 605.1b call.
#[test]
fn bubbling_muck_extra_mana() {
    fn setup() -> (GameState, CardId, usize) {
        let mut g = two_player_game();
        let swamp = land(&mut g, 0, catalog::swamp());
        let muck = g.add_card_to_hand(0, catalog::bubbling_muck());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: muck,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Bubbling Muck");
        drain_stack(&mut g);
        (g, swamp, 0)
    }
    both_ways("bubbling muck", true, setup);
    let (mut g, id, _) = setup();
    tap(&mut g, id, 0);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 2);
}

/// Mana Reflection sets the mana-static lane: generic path.
#[test]
fn mana_static_declines() {
    both_ways("mana reflection", false, || {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::mana_reflection());
        let id = land(&mut g, 0, catalog::forest());
        (g, id, 0)
    });
}

/// A land-type rewrite in scope: a basic's intrinsic ability goes generic
/// (CR 305.6), a land whose printed text is real rules text does not.
#[test]
fn land_type_rewrite() {
    both_ways("urborg + forest", false, || {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::urborg_tomb_of_yawgmoth());
        let id = land(&mut g, 0, catalog::forest());
        (g, id, 0)
    });
    both_ways("urborg + wastes", true, || {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::urborg_tomb_of_yawgmoth());
        let id = land(&mut g, 0, catalog::wastes());
        (g, id, 0)
    });
    // A printed mana ability is real rules text: only a basic's intrinsic one
    // is CR 305.6-gated in `activate_ability_inner`, so both paths accept.
    both_ways("blood moon + temple", true, || {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::blood_moon());
        let id = land(&mut g, 0, catalog::temple_of_epiphany());
        (g, id, 0)
    });
}

#[test]
fn tapped_and_foreign_lands_decline() {
    both_ways("tapped forest", false, || {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::forest());
        g.battlefield_find_mut(id).unwrap().tapped = true;
        (g, id, 0)
    });
    both_ways("opponent's forest", false, || {
        let mut g = two_player_game();
        let id = land(&mut g, 1, catalog::forest());
        (g, id, 0)
    });
}

/// The whole bot loop, both ways: a fixed-seed game must trace identically.
#[test]
fn bot_game_identical_both_ways() {
    use crabomination::cube::CardFactory;
    use crabomination::recommend::trace_game;
    let red: Vec<CardFactory> = [
        (catalog::mountain as CardFactory, 17),
        (catalog::lightning_bolt as CardFactory, 4),
        (catalog::goblin_guide as CardFactory, 4),
        (catalog::gray_ogre as CardFactory, 4),
        (catalog::hill_giant as CardFactory, 4),
    ]
    .iter()
    .flat_map(|&(f, n)| std::iter::repeat_n(f, n))
    .collect();
    let green: Vec<CardFactory> = [
        (catalog::forest as CardFactory, 17),
        (catalog::grizzly_bears as CardFactory, 8),
        (catalog::giant_growth as CardFactory, 4),
    ]
    .iter()
    .flat_map(|&(f, n)| std::iter::repeat_n(f, n))
    .collect();
    FORCE_GENERIC_ACTIVATION.store(false, Relaxed);
    let before = PLAIN_LAND_TAPS.load(Relaxed);
    let fast = trace_game(&red, &green, 0xA11CE, 4_000);
    assert!(PLAIN_LAND_TAPS.load(Relaxed) > before, "no land tap took the fast path");
    FORCE_GENERIC_ACTIVATION.store(true, Relaxed);
    let slow = trace_game(&red, &green, 0xA11CE, 4_000);
    FORCE_GENERIC_ACTIVATION.store(false, Relaxed);
    let at = fast.lines.iter().zip(&slow.lines).position(|(x, y)| x != y);
    assert!(
        at.is_none() && fast.lines.len() == slow.lines.len(),
        "the bot game diverges between the two paths at {at:?}:\n  fast: {:?}\n  slow: {:?}",
        at.and_then(|i| fast.lines.get(i)),
        at.and_then(|i| slow.lines.get(i)),
    );
}
