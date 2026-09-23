//! CR 614 — "If it would leave the battlefield, exile it instead of putting
//! it anywhere else" (`Effect::ExileIfLeavesBattlefield`): a replacement
//! bound to one object, covering every exit (death, bounce), spent when that
//! object leaves (CR 400.7). It used to be a finality counter (Geth), an
//! end-of-turn death redirect (Gruesome Encore), or nothing (Whip of Erebos,
//! Llanowar Greenwidow), so a bounce kept the creature.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// Geth reanimates a Bears under the replacement; returns (game, bears).
fn geth_returns_bears() -> (GameState, CardId) {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let geth = g.add_card_to_battlefield(0, catalog::geth_thane_of_contracts());
    g.battlefield_find_mut(geth).unwrap().summoning_sick = false;
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: geth,
        ability_index: 0,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("Geth reanimates");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some());
    (g, bears)
}

fn cast(g: &mut GameState, id: CardId, target: CardId) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn in_exile(g: &GameState, id: CardId) -> bool {
    g.exile.iter().any(|c| c.id == id)
}

#[test]
fn cr_614_a_reanimated_creature_that_dies_is_exiled() {
    let (mut g, bears) = geth_returns_bears();
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 3);
    cast(&mut g, murder, bears);
    assert!(in_exile(&g, bears));
    assert!(g.players[0].graveyard.iter().all(|c| c.id != bears));
}

#[test]
fn cr_614_a_bounced_one_is_exiled_too_and_the_replacement_is_spent() {
    let (mut g, bears) = geth_returns_bears();
    let unsummon = g.add_card_to_hand(0, catalog::unsummon());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, unsummon, bears);
    assert!(in_exile(&g, bears), "not back in hand");
    assert!(g.players[0].hand.iter().all(|c| c.id != bears));
    // CR 400.7 — whatever comes back later is a new object.
    assert!(g.replacement_effects.is_empty(), "the object-bound replacement lapsed");
}
