//! CR 702.55 — Haunt. Functionality tests for the Guildpact haunt cards in
//! `catalog::sets::gpt`.

use crate::catalog;
use crate::game::*;
use crate::mana::Color;

/// A haunt creature is exiled (not graveyard'd) when it dies, then its haunt
/// body fires when the haunted creature dies.
#[test]
fn shrieking_grotesque_haunts_then_payoff_on_death() {
    let mut g = two_player_game();
    let grotesque = g.add_card_to_battlefield(0, catalog::shrieking_grotesque());
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4, survives
    g.add_card_to_hand(1, catalog::grizzly_bears()); // the one card to discard

    // Kill the Grotesque → it's exiled haunting the opponent's creature.
    g.battlefield_find_mut(grotesque).unwrap().damage = 1; // lethal vs 2/1
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == grotesque), "exiled haunting");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == grotesque), "not in graveyard");
    assert_eq!(g.players[1].hand.len(), 1, "payoff not fired yet");

    // The haunted creature dies → opponent discards a card.
    g.battlefield_find_mut(foe).unwrap().damage = 4;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "haunt payoff: opponent discarded");
}

/// Mourning Thrull's gain-2-and-draw trigger fires on entry.
#[test]
fn mourning_thrull_etb_gain_and_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::mourning_thrull());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "ETB gained 2");
    assert_eq!(g.players[0].hand.len(), hand + 1, "ETB drew a card");
}

/// Mourning Thrull's haunt body (gain 2, draw 1) fires when the haunted
/// creature dies, even though the Thrull itself is in exile.
#[test]
fn mourning_thrull_haunt_payoff_on_haunted_death() {
    let mut g = two_player_game();
    let thrull = g.add_card_to_battlefield(0, catalog::mourning_thrull());
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    for _ in 0..2 { g.add_card_to_library(0, catalog::grizzly_bears()); }

    g.battlefield_find_mut(thrull).unwrap().damage = 1; // lethal vs 1/1
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == thrull));

    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.battlefield_find_mut(foe).unwrap().damage = 4;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "haunt gained 2");
    assert_eq!(g.players[0].hand.len(), hand + 1, "haunt drew a card");
}

/// A haunt instant resolves its main effect, is exiled haunting a creature
/// (not graveyard'd), then fires its haunt body when that creature dies.
#[test]
fn douse_in_gloom_instant_haunts() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let douse = g.add_card_to_hand(0, catalog::douse_in_gloom());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);

    let life = g.players[0].life;
    cast_at(&mut g, douse, Target::Permanent(foe));
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 2, "dealt 2");
    assert_eq!(g.players[0].life, life + 2, "gained 2");
    assert!(g.exile.iter().any(|c| c.id == douse), "spell exiled haunting");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == douse), "not graveyard'd");

    // Kill the haunted creature → haunt body: 2 to the opponent, gain 2.
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.battlefield_find_mut(foe).unwrap().damage = 4;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2, "haunt dealt 2 to opponent");
    assert_eq!(g.players[0].life, p0_life + 2, "haunt gained 2");
}

/// Castigate exiles a nonland from the opponent's hand on cast and again when
/// the haunted creature dies.
#[test]
fn castigate_haunt_repeats_hand_exile() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let cast_id = g.add_card_to_hand(0, catalog::castigate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    cast_at(&mut g, cast_id, Target::Player(1));
    assert_eq!(g.players[1].hand.len(), 1, "cast exiled one nonland");

    g.battlefield_find_mut(foe).unwrap().damage = 4;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "haunt exiled the second nonland");
}
