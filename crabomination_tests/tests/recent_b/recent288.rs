//! Functionality tests for `catalog::sets::decks::recent288` — Doc Aurlock's
//! graveyard/exile/plot cost reductions.

use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameState};
use crabomination::mana::Color;
use crabomination::TurnStep;

/// Doc Aurlock reduces Plot activation costs by {2}: Longhorn Sharpshooter's
/// {3}{R} plot cost becomes {1}{R}.
#[test]
fn doc_aurlock_discounts_plot() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::doc_aurlock_grizzled_genius());
    let card = g.add_card_to_hand(0, catalog::longhorn_sharpshooter());
    // Only {1}{R} available — the full {3}{R} would be short.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Plot { card_id: card }).expect("plot at the reduced cost");
    assert!(g.exile.iter().any(|c| c.id == card), "the plotted card sits in exile");
    assert_eq!(g.players[0].mana_pool.total(), 0, "the reduced cost drained the pool exactly");
}

/// Doc Aurlock reduces exile casts by {2}: a foretold Behold the Multiverse
/// (foretell {1}{U}) casts for just {U}.
#[test]
fn doc_aurlock_discounts_exile_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::doc_aurlock_grizzled_genius());
    let card = g.add_card_to_exile(0, catalog::behold_the_multiverse());
    g.exile.iter_mut().find(|c| c.id == card).unwrap().face_down = true;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastForetold {
        card_id: card,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the foretold spell at the reduced exile cost");
    assert_eq!(g.players[0].mana_pool.total(), 0, "only one blue mana was spent");
}

fn ready(g: &mut GameState) {
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
}

/// Saddling Fortune records the rider in `saddled_by`.
#[test]
fn fortune_records_saddlers() {
    let mut g = two_player_game();
    let fortune = g.add_card_to_battlefield(0, catalog::fortune_loyal_steed());
    let rider = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fortune);
    g.clear_sickness(rider);
    ready(&mut g);
    g.perform_action(GameAction::Saddle { mount: fortune, creatures: vec![rider] }).expect("saddle");
    let m = g.battlefield_find(fortune).unwrap();
    assert!(m.saddled, "Fortune is saddled");
    assert_eq!(m.saddled_by, vec![rider], "the rider is remembered");
}

/// Fortune attacks while saddled → at end of combat it and one saddler blink,
/// returning untapped and summoning-sick.
#[test]
fn fortune_end_of_combat_blink() {
    let mut g = two_player_game();
    let fortune = g.add_card_to_battlefield(0, catalog::fortune_loyal_steed());
    let rider = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fortune);
    g.clear_sickness(rider);
    ready(&mut g);
    g.perform_action(GameAction::Saddle { mount: fortune, creatures: vec![rider] }).expect("saddle");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: fortune, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    // End of combat → the delayed blink fires.
    g.fire_step_triggers(TurnStep::EndCombat);
    drain_stack(&mut g);
    for id in [fortune, rider] {
        let c = g.battlefield_find(id).expect("returned to the battlefield");
        assert!(!c.tapped, "returns untapped");
        assert!(c.summoning_sick, "returns as a fresh, summoning-sick object");
        assert!(!c.saddled, "the returned Mount is no longer saddled");
    }
}
