//! Functionality tests for `catalog::sets::decks::recent176`.

use crate::catalog;
use crate::game::*;
use crate::mana::Color;

/// Dune Drifter's ETB *triggered* ability reads the cast's X: cast with X=2 and
/// a mana-value-2 card in the graveyard returns to the battlefield.
#[test]
fn dune_drifter_etb_reads_cast_x() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // {1}{G} = MV 2
    let spell = g.add_card_to_hand(0, catalog::dune_drifter());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2); // X=2
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast Dune Drifter with X=2");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.id == dead),
        "MV-2 creature reanimated by X=2 ETB"
    );
}

/// With X=1 the same MV-2 card is not a legal target, so nothing returns.
#[test]
fn dune_drifter_x_gate_excludes_larger_card() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let spell = g.add_card_to_hand(0, catalog::dune_drifter());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1); // X=1
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(1),
    })
    .expect("cast Dune Drifter with X=1");
    drain_stack(&mut g);
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == dead),
        "MV-2 card stays in graveyard when X=1"
    );
}
