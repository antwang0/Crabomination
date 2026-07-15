//! Functionality tests for the `catalog::sets::decks::recent15` Elf batch.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::*;

/// Shaman of the Pack drains the opponent for the number of Elves you control.
#[test]
fn shaman_of_the_pack_drains_by_elf_count() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::llanowar_elves()); // an Elf
    let life = g.players[1].life;
    let id = g.add_card_to_battlefield(0, catalog::shaman_of_the_pack()); // a second Elf
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    // Two Elves you control (Llanowar + Shaman) → opponent loses 2.
    assert_eq!(g.players[1].life, life - 2);
}

/// Elvish Warmaster mints an Elf token when another Elf enters (once per turn).
#[test]
fn elvish_warmaster_makes_token_on_elf_entry() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::elvish_warmaster());
    // Cast the Elf so its entry dispatches to the Warmaster's watcher trigger.
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.priority.player_with_priority = 0;
    cast(&mut g, elf);
    drain_stack(&mut g);
    let tokens = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Elf Warrior" && c.is_token).count();
    assert_eq!(tokens, 1, "one Elf Warrior token minted");
}

/// Elvish Warmaster's activated ability pumps Elves and grants deathtouch.
#[test]
fn elvish_warmaster_anthem_grants_deathtouch() {
    let mut g = two_player_game();
    let warmaster = g.add_card_to_battlefield(0, catalog::elvish_warmaster());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: warmaster, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate anthem");
    drain_stack(&mut g);
    let cp = g.computed_permanent(warmaster).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::Deathtouch));
}
