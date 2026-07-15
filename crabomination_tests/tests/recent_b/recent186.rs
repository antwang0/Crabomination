//! Functionality tests for `catalog::sets::decks::recent186` (DSK/BLB gaps).

use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::*;
use crabomination::mana::Color;

/// Vanish from Sight tucks a nonland permanent into its owner's library and
/// surveils.
#[test]
fn vanish_from_sight_tucks_permanent() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(1, catalog::howling_mine());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::vanish_from_sight());
    g.add_card_to_library(0, catalog::grizzly_bears()); // to surveil
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let opp_lib_before = g.players[1].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Vanish from Sight");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "permanent left the battlefield");
    assert_eq!(g.players[1].library.len(), opp_lib_before + 1, "tucked into owner's library");
}

/// Hearthborn Battler pings the opponent on any player's second spell.
#[test]
fn hearthborn_battler_pings_on_second_spell() {
    let mut g = two_player_game();
    let _battler = g.add_card_to_battlefield(0, catalog::hearthborn_battler());
    let s1 = g.add_card_to_hand(0, catalog::divination());
    let s2 = g.add_card_to_hand(0, catalog::divination());
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.players[1].life = 20;
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: s1, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("first spell");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20, "no ping on the first spell");
    g.perform_action(GameAction::CastSpell {
        card_id: s2, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("second spell");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "second spell pinged the opponent for 2");
}

/// Inquisitive Glimmer makes enchantment spells cost {1} less.
#[test]
fn inquisitive_glimmer_discounts_enchantments() {
    let cast_with_only_red = |glimmer: bool| -> bool {
        let mut g = two_player_game();
        if glimmer {
            g.add_card_to_battlefield(0, catalog::inquisitive_glimmer());
        }
        let bomb = g.add_card_to_hand(0, catalog::goblin_bombardment()); // {1}{R}
        g.players[0].mana_pool.add(Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: bomb, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .is_ok()
    };
    assert!(!cast_with_only_red(false), "{{1}}{{R}} unpayable with only {{R}}");
    assert!(cast_with_only_red(true), "Glimmer's -{{1}} makes it castable for {{R}}");
}

/// Tidecaller Mentor bounces a permanent only when threshold is active.
#[test]
fn tidecaller_mentor_threshold_bounce() {
    let bounced = |gy: usize| -> bool {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        for _ in 0..gy {
            g.add_card_to_graveyard(0, catalog::grizzly_bears());
        }
        g.move_card_to_battlefield_for_test(0, catalog::tidecaller_mentor());
        drain_stack(&mut g);
        g.battlefield_find(victim).is_none()
    };
    assert!(!bounced(6), "below threshold → no bounce");
    assert!(bounced(7), "threshold met → bounced a permanent");
}

/// Thought-Stalker Warlock's ETB forces a discard, targeting the opponent's hand
/// when they lost life this turn.
#[test]
fn thought_stalker_warlock_conditional_discard() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    // Opponent lost life this turn → targeted discard.
    g.adjust_life(1, -1);
    let opp_hand_before = g.players[1].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::thought_stalker_warlock());
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1, "opponent discarded a card");
}
