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

/// Vnwxt doubles draws only at max speed (4).
#[test]
fn vnwxt_draw_doubles_at_max_speed() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vnwxt_verbose_host());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let mut events = Vec::new();
    // Below max: a draw yields exactly one card.
    g.players[0].speed = 3;
    let before = g.players[0].hand.len();
    g.draw_one(0, &mut events);
    assert_eq!(g.players[0].hand.len(), before + 1, "speed 3 → single draw");
    // Max speed: a draw yields two cards.
    g.players[0].speed = 4;
    let before = g.players[0].hand.len();
    g.draw_one(0, &mut events);
    assert_eq!(g.players[0].hand.len(), before + 2, "speed 4 → doubled draw");
}

/// Zahur's max-speed death trigger mints a tapped Zombie; below max, nothing.
#[test]
fn zahur_max_speed_death_makes_zombie() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::zahur_glorys_past());
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].speed = 4;
    g.battlefield_find_mut(victim).unwrap().damage = 99; // lethal → CreatureDied
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0
            && c.definition.name == "Zombie"
            && c.tapped),
        "max speed minted a tapped Zombie on the death"
    );
}

/// Zahur's sac ability surveils and is once-per-turn.
#[test]
fn zahur_sac_ability_is_once_per_turn() {
    let mut g = two_player_game();
    let zahur = g.add_card_to_battlefield(0, catalog::zahur_glorys_past());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: zahur,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("sac another creature → surveil");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder was sacrificed");
    // A second activation this turn is rejected (no creature to sac anyway, but
    // the once-per-turn gate fires first).
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let r = g.perform_action(GameAction::ActivateAbility {
        card_id: zahur,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    });
    assert!(r.is_err(), "second activation blocked by once-per-turn");
    assert!(g.battlefield_find(other).is_some(), "no extra creature sacrificed");
}

/// The Last Ride shrinks by your life total and its {2}{B}, pay-2-life ability
/// draws a card.
#[test]
fn the_last_ride_scales_with_life_and_draws() {
    let mut g = two_player_game();
    let ride = g.add_card_to_battlefield(0, catalog::the_last_ride());
    g.add_card_to_library(0, catalog::grizzly_bears());
    // At 7 life the 13/13 base reads as 6/6.
    g.players[0].life = 7;
    let cp = g.computed_permanent(ride).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "13/13 − life(7) = 6/6");
    // Pay {2}{B} + 2 life to draw.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ride,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("pay 2 life + {2}{B}: draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 5, "paid 2 life");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// The Speed Demon's end step draws and loses life equal to your speed.
#[test]
fn the_speed_demon_end_step_scales_with_speed() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_speed_demon());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[0].speed = 3;
    g.active_player_idx = 0;
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 3, "drew 3 (speed)");
    assert_eq!(g.players[0].life, life - 3, "lost 3 (speed)");
}
