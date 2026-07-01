//! Functionality tests for `catalog::sets::decks::recent67` — OTJ wave 2.

use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;
use crate::TurnStep;

fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == controller && c.definition.name == name).count()
}

#[test]
fn nezumi_linkbreaker_dies_into_a_mercenary() {
    let mut g = two_player_game();
    let nz = g.add_card_to_battlefield(0, catalog::nezumi_linkbreaker());
    g.remove_to_graveyard_with_triggers(nz);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Mercenary"), 1);
}

#[test]
fn gold_rush_makes_treasure_and_pumps_per_treasure() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::gold_rush());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, id, Target::Permanent(bear));
    // One Treasure made → +2/+2 → 4/4.
    assert_eq!(count_named(&g, 0, "Treasure"), 1);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

#[test]
fn prosperity_tycoon_etb_mercenary_and_sac_for_indestructible() {
    let mut g = two_player_game();
    let pt = g.add_card_to_battlefield(0, catalog::prosperity_tycoon());
    g.fire_self_etb_triggers(pt, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Mercenary"), 1, "ETB made a Mercenary");
    // Sac the token for indestructible.
    g.clear_sickness(pt);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pt, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate sac-token ability");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Mercenary"), 0, "the token was sacrificed");
    assert!(
        g.computed_permanent(pt).unwrap().keywords.contains(&crate::card::Keyword::Indestructible)
    );
}

#[test]
fn iron_fist_pulverizer_burns_on_second_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::iron_fist_pulverizer());
    g.players[1].life = 20;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // First spell: no trigger.
    let s1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s1, Target::Player(1));
    // Second spell triggers Iron-Fist for 2 to the opponent (auto-targeted).
    let s2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s2, Target::Player(1));
    // Bolt 1 (3) + Bolt 2 (3) + Iron-Fist (2) = 8.
    assert_eq!(g.players[1].life, 12);
}
