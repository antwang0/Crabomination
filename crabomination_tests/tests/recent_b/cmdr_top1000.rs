//! Top-1000 Commander staples (`decks::cmdr_top1000`, COMMANDER_BACKLOG §2).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// CR 614.1a: Xorn replaces a Treasure creation with that many plus one.
#[test]
fn xorn_adds_a_treasure() {
    let mut g = pod(4);
    g.add_card_to_battlefield(0, catalog::xorn());
    let s = g.add_card_to_hand(0, catalog::strike_it_rich());
    cast(&mut g, 0, s, None).unwrap();
    assert_eq!(count_named(&g, 0, "Treasure"), 2);
}

/// CR 614.1a: Peregrin Took adds one Food to any token creation, once per
/// event; sacrificing three Foods draws a card.
#[test]
fn peregrin_took_adds_a_food_and_eats_three() {
    let mut g = pod(4);
    let pip = g.add_card_to_battlefield(0, catalog::peregrin_took());
    let s = g.add_card_to_hand(0, catalog::raise_the_alarm());
    cast(&mut g, 0, s, None).unwrap();
    assert_eq!(count_named(&g, 0, "Soldier"), 2);
    assert_eq!(count_named(&g, 0, "Food"), 1, "one extra Food per creation event");
    for _ in 0..2 {
        let s = g.add_card_to_hand(0, catalog::raise_the_alarm());
        cast(&mut g, 0, s, None).unwrap();
    }
    assert_eq!(count_named(&g, 0, "Food"), 3);
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: pip, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("sacrifice three Foods");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Food"), 0);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// CR 613.4c: Flowering of the White Tree — legendary creatures +2/+1 and
/// ward {1}, nonlegendary +1/+1; an opponent's creature is untouched.
#[test]
fn flowering_of_the_white_tree_splits_legendary_and_not() {
    let mut g = pod(4);
    g.add_card_to_battlefield(0, catalog::flowering_of_the_white_tree());
    let leg = g.add_card_to_battlefield(0, catalog::lotho_corrupt_shirriff()); // 2/1
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let view = g.compute_battlefield();
    let get = |id| view.iter().find(|c| c.id == id).unwrap();
    assert_eq!((get(leg).power, get(leg).toughness), (4, 2));
    assert!(get(leg).keywords().iter().any(|k| matches!(k, Keyword::Ward(_))));
    assert_eq!((get(bear).power, get(bear).toughness), (3, 3));
    assert!(!get(bear).keywords().iter().any(|k| matches!(k, Keyword::Ward(_))));
    assert_eq!((get(theirs).power, get(theirs).toughness), (2, 2));
}

/// Lotho triggers on any player's second spell of the turn (not the first
/// or third): its controller loses 1 and makes a Treasure.
#[test]
fn lotho_triggers_on_each_players_second_spell() {
    let mut g = pod(4);
    g.add_card_to_battlefield(0, catalog::lotho_corrupt_shirriff());
    let life = g.players[0].life;
    for n in 1..=3 {
        let s = g.add_card_to_hand(2, catalog::opt());
        cast(&mut g, 2, s, None).unwrap();
        let expect = if n >= 2 { 1 } else { 0 };
        assert_eq!(count_named(&g, 0, "Treasure"), expect, "after spell {n}");
    }
    assert_eq!(g.players[0].life, life - 1);
}

/// CR 702.8a: Borne Upon a Wind lets its caster cast a sorcery at instant
/// speed this turn, and draws.
#[test]
fn borne_upon_a_wind_grants_flash_and_draws() {
    let mut g = pod(4);
    g.add_card_to_library(0, catalog::island());
    let b = g.add_card_to_hand(0, catalog::borne_upon_a_wind());
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, b, None).unwrap();
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    g.step = TurnStep::End;
    let sorc = g.add_card_to_hand(0, catalog::strike_it_rich());
    cast(&mut g, 0, sorc, None).expect("a sorcery in the end step");
    assert_eq!(count_named(&g, 0, "Treasure"), 1);
}

/// CR 115.7a: Imp's Mischief redirects a Bolt from its caster's face to
/// another opponent and costs life equal to the Bolt's mana value.
#[test]
fn imps_mischief_redirects_and_costs_mana_value_in_life() {
    let mut g = pod(4);
    g.priority.player_with_priority = 1;
    flood(&mut g, 1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt seat 0");
    let imp = g.add_card_to_hand(0, catalog::imps_mischief());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Target(Target::Player(1)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: imp, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Imp's Mischief");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19, "lost 1 (the Bolt's mana value), not 3");
    assert_eq!(g.players[1].life, 17, "the Bolt went elsewhere");
}

/// CR 603.4: Boromir counters an opponent's spell cast for no mana, and
/// lets a paid one resolve.
#[test]
fn boromir_counters_only_free_spells() {
    let mut g = pod(4);
    g.add_card_to_battlefield(0, catalog::boromir_warden_of_the_tower());
    let paid = g.add_card_to_hand(1, catalog::raise_the_alarm());
    cast(&mut g, 1, paid, None).unwrap();
    assert_eq!(count_named(&g, 1, "Soldier"), 2, "a paid spell resolves");
    g.active_player_idx = 1;
    let free = g.add_card_to_hand(1, catalog::ornithopter());
    cast(&mut g, 1, free, None).unwrap();
    assert_eq!(count_named(&g, 1, "Ornithopter"), 0, "a {{0}} spell is countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == free));
}

/// Jaheira: a token you control taps for {G}; an opponent's doesn't gain it.
#[test]
fn jaheira_gives_your_tokens_a_green_mana_ability() {
    let mut g = pod(4);
    g.add_card_to_battlefield(0, catalog::jaheira_friend_of_the_forest());
    let s = g.add_card_to_hand(0, catalog::strike_it_rich());
    cast(&mut g, 0, s, None).unwrap();
    let t = g.battlefield.iter().find(|c| c.definition.name == "Treasure").unwrap().id;
    g.players[0].mana_pool = Default::default();
    // The Treasure's own ability is index 0; the granted {T}: Add {G} is next.
    g.perform_action(GameAction::ActivateAbility {
        card_id: t, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("granted mana ability");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    assert!(g.battlefield.iter().any(|c| c.id == t), "tapped, not sacrificed");
}

/// Torment of Hailfire, X = 3, in a four-seat pod: an opponent at 5 life
/// pays 3 once, then must discard, then sacrifice; the other opponents
/// just pay 9 (CR 107.3 — X is the announced value).
#[test]
fn torment_of_hailfire_punishes_each_opponent_x_times() {
    let mut g = pod(4);
    g.players[1].life = 5;
    g.add_card_to_hand(1, catalog::island());
    let keep = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::torment_of_hailfire());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: t, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("cast Torment");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 2, "paid once, above zero");
    assert!(g.players[1].hand.is_empty(), "then discarded");
    assert!(!g.battlefield.iter().any(|c| c.id == keep), "then sacrificed");
    let start = g.players[0].life;
    assert_eq!(g.players[2].life, start - 9);
    assert_eq!(g.players[3].life, start - 9);
}
