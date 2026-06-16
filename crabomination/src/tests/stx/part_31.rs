//! Functionality tests for the Mystical Archive (STA) batch in
//! `catalog::sets::stx::sta`: Infuriate, Blue Sun's Zenith, Abundant Harvest,
//! Urza's Rage, Natural Order.

use crate::catalog;
use crate::game::two_player_game;
use crate::mana::Color;
use super::*;

/// Infuriate pumps the target +3/+2 until end of turn.
#[test]
fn infuriate_pumps_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::infuriate());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Infuriate");
    drain_stack(&mut g);

    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (5, 4), "+3/+2 EOT");
}

/// Blue Sun's Zenith draws X for the target player and shuffles itself back
/// into the library (not the graveyard).
#[test]
fn blue_suns_zenith_draws_x_and_shuffles_back() {
    let mut g = two_player_game();
    for _ in 0..5 {
        let id = g.next_id();
        g.players[0].add_to_library_top(id, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::blue_suns_zenith());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast Blue Sun's Zenith");
    drain_stack(&mut g);

    // Cast removes it from hand (-1), then draws 2 (+2) → net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew 2, cast left hand");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != spell), "not in graveyard");
    assert!(g.players[0].library.iter().any(|c| c.id == spell), "shuffled into library");
}

/// Abundant Harvest (land mode auto-picked) digs to the first land and puts it
/// into hand; the nonland miss is bottomed.
#[test]
fn abundant_harvest_finds_a_land() {
    let mut g = two_player_game();
    // Top → bottom: bear (nonland miss), Forest (the find).
    let forest = g.next_id();
    g.players[0].add_to_library_top(forest, catalog::forest());
    let bear = g.next_id();
    g.players[0].add_to_library_top(bear, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::abundant_harvest());
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Abundant Harvest");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "land put into hand");
    assert!(g.players[0].hand.iter().all(|c| c.id != bear), "nonland miss not in hand");
}

/// Urza's Rage deals 3 unkicked, 10 kicked, and can't be countered.
#[test]
fn urzas_rage_kicked_deals_ten() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::urzas_rage());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(10); // {2}{R} + kicker {8}{R} = {10}{R}{R}
    let life = g.players[1].life;

    g.perform_action(GameAction::CastSpellKicked {
        card_id: spell,
        target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked Urza's Rage");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life - 10, "kicked → 10 damage");
}

/// Natural Order sacrifices a green creature and fetches a green creature onto
/// the battlefield.
#[test]
fn natural_order_sacrifices_and_fetches() {
    let mut g = two_player_game();
    let sac = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green creature to sac
    let big = g.next_id();
    g.players[0].add_to_library_top(big, catalog::craw_wurm()); // green creature in library
    let spell = g.add_card_to_hand(0, catalog::natural_order());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    // The library search is declined by the AutoDecider; script the pick.
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(big)),
    ]));

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Natural Order");
    drain_stack(&mut g);

    assert!(g.battlefield_find(sac).is_none(), "green creature sacrificed as cost");
    assert!(g.battlefield_find(big).is_some(), "fetched creature on battlefield");
}
