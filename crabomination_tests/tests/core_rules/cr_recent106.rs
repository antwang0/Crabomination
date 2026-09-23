//! CR 508.4 — a creature put onto the battlefield attacking was never
//! declared as an attacker, so "whenever ~ attacks" abilities don't trigger
//! for it. General Kreat's "whenever a Goblin you control attacks, create a
//! Goblin that's tapped and attacking" used to fire again for each token it
//! made: 922 Goblins on one seat by turn 57 of a 15-seat pod game.

use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

#[test]
fn cr_508_4_a_token_entering_attacking_does_not_trigger_attacks() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let kreat = g.add_card_to_battlefield(0, catalog::general_kreat_the_boltbringer());
    g.clear_sickness(kreat);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: kreat,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let goblins: Vec<_> =
        g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Goblin").map(|c| c.id).collect();
    assert_eq!(goblins.len(), 1, "one declared Goblin, one token");
    let tok = goblins[0];
    assert!(g.battlefield_find(tok).unwrap().tapped);
    assert!(g.attacking().iter().any(|a| a.attacker == tok), "still an attacking creature");
}
