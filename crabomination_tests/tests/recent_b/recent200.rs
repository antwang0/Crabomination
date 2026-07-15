//! Functionality tests for `catalog::sets::decks::recent200`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};

/// Dragonfire Blade grants +2/+2 and hexproof from monocolored to its bearer.
#[test]
fn dragonfire_blade_equips_and_buffs() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bearer = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let blade = g.add_card_to_battlefield(0, catalog::dragonfire_blade());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::Equip { equipment: blade, target: bearer })
        .expect("equip Dragonfire Blade");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bearer).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2 from the Blade");
    assert!(cp.keywords.contains(&Keyword::HexproofFromMonocolored), "granted hexproof from monocolored");
}
