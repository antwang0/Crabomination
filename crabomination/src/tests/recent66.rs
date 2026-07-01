//! Functionality tests for `catalog::sets::decks::recent66` — OTJ staples.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::mana::Color;
use crate::TurnStep;

fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == controller && c.definition.name == name).count()
}

/// Cast Lava Spike at the opponent to commit a crime.
fn commit_crime(g: &mut GameState) {
    let ls = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(g, ls, Target::Player(1));
}

fn attack_saddled(g: &mut GameState, id: CardId) {
    g.battlefield_find_mut(id).unwrap().saddled = true;
    g.clear_sickness(id);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(g);
}

#[test]
fn vengeful_townsfolk_grows_when_others_die() {
    let mut g = two_player_game();
    let vt = g.add_card_to_battlefield(0, catalog::vengeful_townsfolk());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Kill the ally through the full damage→SBA→dispatch path.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, bolt, Target::Permanent(ally));
    assert_eq!(g.battlefield_find(vt).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn loan_shark_draws_when_two_spells_cast() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[0].spells_cast_this_turn = 2;
    let ls = g.add_card_to_battlefield(0, catalog::loan_shark());
    let hand = g.players[0].hand.len();
    g.fire_self_etb_triggers(ls, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "ETB drew with 2 spells cast");
    assert!(catalog::loan_shark().plot_cost.is_some());
}

#[test]
fn loan_shark_no_draw_below_threshold() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].spells_cast_this_turn = 1;
    let ls = g.add_card_to_battlefield(0, catalog::loan_shark());
    let hand = g.players[0].hand.len();
    g.fire_self_etb_triggers(ls, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "one spell isn't enough");
}

#[test]
fn rattleback_apothecary_grants_menace_on_crime() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rattleback_apothecary());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Mode 0 = menace.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![0])]));
    commit_crime(&mut g);
    assert!(g
        .computed_permanent(bear)
        .unwrap()
        .keywords
        .contains(&Keyword::Menace));
}

#[test]
fn servant_of_the_stinger_has_deathtouch() {
    let d = catalog::servant_of_the_stinger();
    assert!(d.keywords.contains(&Keyword::Deathtouch));
    assert_eq!((d.power, d.toughness), (1, 3));
}

#[test]
fn deserts_due_scales_with_deserts() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    // Two Deserts → -2/-2 plus -2/-2 = -4/-4 → 0/0, dies.
    g.add_card_to_battlefield(0, catalog::conduit_pylons());
    g.add_card_to_battlefield(0, catalog::conduit_pylons());
    let id = g.add_card_to_hand(0, catalog::deserts_due());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, id, Target::Permanent(victim));
    assert!(g.battlefield_find(victim).is_none(), "-4/-4 killed the 4/4");
}

#[test]
fn deserts_due_base_without_deserts() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::deserts_due());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, id, Target::Permanent(victim));
    let cp = g.computed_permanent(victim).expect("survives -2/-2");
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

#[test]
fn quick_draw_pumps_own_and_strips_opponent() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::combat_thresher()); // has double strike
    let id = g.add_card_to_hand(0, catalog::quick_draw());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    })
    .expect("cast Quick Draw");
    drain_stack(&mut g);
    let cp = g.computed_permanent(mine).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
    assert!(
        !g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "opponent's double strike stripped"
    );
}

#[test]
fn prickly_pair_makes_a_mercenary() {
    let mut g = two_player_game();
    let pp = g.add_card_to_battlefield(0, catalog::prickly_pair());
    g.fire_self_etb_triggers(pp, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Mercenary"), 1);
}

#[test]
fn bounding_felidar_buffs_team_when_saddled_attacks() {
    let mut g = two_player_game();
    let felidar = g.add_card_to_battlefield(0, catalog::bounding_felidar());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    attack_saddled(&mut g, felidar);
    assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].life, life + 1, "gained 1 per other creature");
}

#[test]
fn trained_arynx_first_strike_when_saddled_attacks() {
    let mut g = two_player_game();
    let arynx = g.add_card_to_battlefield(0, catalog::trained_arynx());
    attack_saddled(&mut g, arynx);
    assert!(g.computed_permanent(arynx).unwrap().keywords.contains(&Keyword::FirstStrike));
}

fn cast_weatherseed(g: &mut GameState) -> CardId {
    let id = g.add_card_to_hand(0, catalog::the_weatherseed_treaty());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast The Weatherseed Treaty");
    drain_stack(g);
    id
}

#[test]
fn weatherseed_treaty_read_ahead_starts_on_chosen_chapter() {
    let mut g = two_player_game();
    // Read ahead → start on chapter II (make a Saproling), skipping chapter I.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    let saga = cast_weatherseed(&mut g);
    assert_eq!(
        g.battlefield_find(saga).unwrap().counter_count(CounterType::Lore),
        2,
        "entered with 2 lore counters"
    );
    assert_eq!(count_named(&g, 0, "Saproling"), 1, "chapter II fired");
}

#[test]
fn weatherseed_treaty_read_ahead_defaults_to_chapter_one() {
    let mut g = two_player_game();
    // AutoDecider declines the amount → falls back to chapter I.
    let saga = cast_weatherseed(&mut g);
    assert_eq!(
        g.battlefield_find(saga).unwrap().counter_count(CounterType::Lore),
        1,
        "default start is chapter I"
    );
    assert_eq!(count_named(&g, 0, "Saproling"), 0, "chapter II not fired");
}

#[test]
fn frenzy_sliver_grants_frenzy_to_unblocked_slivers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::frenzy_sliver());
    let gale = g.add_card_to_battlefield(0, catalog::galerider_sliver()); // 1/1
    g.clear_sickness(gale);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: gale,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    drain_stack(&mut g);
    // Unblocked → Frenzy 1 from Frenzy Sliver: +1/+0.
    assert_eq!(g.computed_permanent(gale).unwrap().power, 2);
}

#[test]
fn rambling_possum_pumps_when_saddled_attacks() {
    let mut g = two_player_game();
    let possum = g.add_card_to_battlefield(0, catalog::rambling_possum());
    attack_saddled(&mut g, possum);
    let cp = g.computed_permanent(possum).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 5), "+1/+2 while saddled");
}
