//! Functionality tests for `catalog::sets::decks::recent211`.

use crate::catalog;
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Fynn turns any deathtouch creature's combat damage into two poison counters.
#[test]
fn fynn_grants_poison_on_deathtouch_hit() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fynn_the_fangbearer());
    // A separate deathtouch attacker — Fynn's ability keys off any deathtoucher.
    let biter = g.add_card_to_battlefield(0, catalog::typhoid_rats());
    g.clear_sickness(biter);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: biter, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 2, "deathtouch hit gave two poison");
}

/// River's Rebuke returns all of a player's nonland permanents to hand.
#[test]
fn rivers_rebuke_bounces_target_players_board() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let art = g.add_card_to_battlefield(1, catalog::feldons_cane());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::rivers_rebuke());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast River's Rebuke");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature bounced");
    assert!(g.battlefield_find(art).is_none(), "artifact bounced");
    assert!(g.battlefield_find(land).is_some(), "land stays");
    assert!(g.battlefield_find(mine).is_some(), "my board untouched");
    assert_eq!(g.players[1].hand.iter().filter(|c| c.id == bear || c.id == art).count(), 2);
}

/// Painful Quandary drains an opponent 5 life when they can't spare a card.
#[test]
fn painful_quandary_drains_on_empty_hand() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::painful_quandary());
    // Opponent casts a bolt from an otherwise-empty hand → can't discard → -5.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a spell");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 15, "no card to discard → lost 5 life");
}

/// Lathliss mints a 5/5 Dragon whenever another nontoken Dragon enters.
#[test]
fn lathliss_mints_dragon_on_dragon_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lathliss_dragon_queen());
    let before = g.battlefield.iter().filter(|c| c.definition.name == "Dragon").count();
    let dm = g.add_card_to_hand(0, catalog::dragon_mage());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: dm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dragon Mage");
    drain_stack(&mut g);
    let after = g.battlefield.iter().filter(|c| c.definition.name == "Dragon" && c.controller == 0).count();
    assert_eq!(after, before + 1, "a 5/5 Dragon token was minted");
}

/// Bolt Bend costs {3} less while you control a creature with power 4+.
#[test]
fn bolt_bend_cost_reduction() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crate::card::CardInstance::new(g.next_id(), catalog::bolt_bend(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "no big creature → no discount");
    g.add_card_to_battlefield(0, catalog::ancestor_dragon()); // 5/6
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 3, "power-4+ creature → {{3}} off");
}
