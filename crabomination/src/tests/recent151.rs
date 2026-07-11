//! Functionality tests for `catalog::sets::decks::recent151`.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::game::two_player_game;
use crate::mana::Color;

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

/// Resurrected Cultist reanimates itself from the graveyard with delirium, with
/// a finality counter.
#[test]
fn resurrected_cultist_delirium_reanimate() {
    let mut g = two_player_game();
    let cultist = g.add_card_to_graveyard(0, catalog::resurrected_cultist());
    // Seed four card types in the graveyard for delirium.
    g.add_card_to_graveyard(0, catalog::forest()); // Land
    g.add_card_to_graveyard(0, catalog::lightning_strike()); // Instant
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // Creature (+ the Cultist)
    g.add_card_to_graveyard(0, catalog::sol_ring()); // Artifact
    g.active_player_idx = 0;
    g.step = crate::game::TurnStep::PreCombatMain;
    fill_mana(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cultist, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("delirium reanimate");
    drain_stack(&mut g);
    let c = g.battlefield_find(cultist).expect("returned to battlefield");
    assert_eq!(c.counter_count(CounterType::Finality), 1, "entered with a finality counter");
}

/// Overgrown Zealot taps for one mana of any color.
#[test]
fn overgrown_zealot_taps_for_any_color() {
    let mut g = two_player_game();
    let zealot = g.add_card_to_battlefield(0, catalog::overgrown_zealot());
    g.clear_sickness(zealot);
    g.perform_action(GameAction::ActivateAbility {
        card_id: zealot, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
    assert!(g.battlefield_find(zealot).unwrap().tapped, "tapped for the mana");
}

/// Gila Courser impulses the top card when it attacks while saddled.
#[test]
fn gila_courser_saddled_attack_impulse() {
    let mut g = two_player_game();
    let courser = g.add_card_to_battlefield(0, catalog::gila_courser());
    g.clear_sickness(courser);
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::grizzly_bears());
    g.battlefield_find_mut(courser).unwrap().saddled = true;
    g.active_player_idx = 0;
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: courser, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == top), "saddled attack impulsed the top card");
}

/// Grab the Prize draws two and burns each opponent when a nonland was discarded.
#[test]
fn grab_the_prize_nonland_discard_burns() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::grizzly_bears()); // nonland to discard
    let id = g.add_card_to_hand(0, catalog::grab_the_prize());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Grab the Prize");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2, "nonland discard dealt 2 to the opponent");
}

/// Malevolent Chandelier bottoms a card from a graveyard.
#[test]
fn malevolent_chandelier_bottoms_graveyard_card() {
    let mut g = two_player_game();
    let chandelier = g.add_card_to_battlefield(0, catalog::malevolent_chandelier());
    g.clear_sickness(chandelier);
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    fill_mana(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: chandelier, ability_index: 0, target: Some(Target::Permanent(corpse)), additional_targets: vec![], x_value: None,
    }).expect("bottom a graveyard card");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().all(|c| c.id != corpse), "left the graveyard");
    assert!(g.players[1].library.iter().any(|c| c.id == corpse), "went to the library");
}

/// Moonstone Harbinger pumps your Bats when you gain life on your turn.
#[test]
fn moonstone_harbinger_life_gain_pumps_bats() {
    let mut g = two_player_game();
    let bat = g.add_card_to_battlefield(0, catalog::moonstone_harbinger());
    g.active_player_idx = 0;
    g.adjust_life(0, 1);
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 1 }]);
    drain_stack(&mut g);
    let c = g.computed_permanent(bat).unwrap();
    assert_eq!(c.power, 2, "Bat got +1/+0 on lifegain");
    assert!(c.keywords.contains(&Keyword::Deathtouch), "still deathtouch");
}
