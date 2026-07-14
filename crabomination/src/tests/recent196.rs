//! Functionality tests for `catalog::sets::decks::recent196`.

use crate::card::CounterType;
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Slickshot Vault-Buster is a 1/4 that swings to 3/4 after a crime.
#[test]
fn slickshot_vault_buster_crime_pump() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::slickshot_vault_buster());
    assert_eq!(g.computed_permanent(id).unwrap().power, 1, "no crime → 1/4");
    g.players[0].committed_crime_this_turn = true;
    assert_eq!(g.computed_permanent(id).unwrap().power, 3, "crime → +2/+0");
}

/// Throw from the Saddle counters a Mount, then it deals its (boosted) power.
#[test]
fn throw_from_the_saddle_mount_counter_and_fight() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let mount = g.add_card_to_battlefield(0, catalog::drover_grizzly()); // 2/2 Bear Mount
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::throw_from_the_saddle());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mount)),
        additional_targets: vec![Target::Permanent(foe)],
        mode: None,
        x_value: None,
    })
    .expect("cast Throw from the Saddle");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(mount).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Mount got a +1/+1 counter",
    );
    assert!(g.battlefield_find(foe).is_none(), "3 power killed the 2/2");
}

/// Shepherd returns a graveyard permanent to hand with no Mount, straight to the
/// battlefield with one.
#[test]
fn shepherd_of_the_clouds_mount_upgrade() {
    // No Mount → returns to hand.
    let mut g = two_player_game();
    let target = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let shep = g.add_card_to_battlefield(0, catalog::shepherd_of_the_clouds());
    g.fire_self_etb_triggers(shep, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == target), "returned to hand");

    // With a Mount → returns to the battlefield.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::drover_grizzly()); // Mount
    let target = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let shep = g.add_card_to_battlefield(0, catalog::shepherd_of_the_clouds());
    g.fire_self_etb_triggers(shep, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_some(), "returned to the battlefield with a Mount");
}

/// Sheriff enters with 1 counter plus one per other creature you control.
#[test]
fn sheriff_scales_with_your_board() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sheriff = g.move_card_to_battlefield_for_test(0, catalog::sheriff_of_safe_passage());
    // 1 base + 2 other creatures = 3 counters → 3/3.
    assert_eq!(
        g.battlefield_find(sheriff).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3,
        "1 + other creatures",
    );
}
