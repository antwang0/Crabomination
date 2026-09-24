//! Commander: the Squirreled Away precon (BLC, Hazel, `decks::cmdr_hazel`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = if n == 2 { two_player_game() } else { multi_player_game(n) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn to_end_step(g: &mut GameState) {
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

/// Deep Forest Hermit: four Squirrels, each 2/2 under its anthem.
#[test]
fn deep_forest_hermit_brings_four_big_squirrels() {
    let mut g = pod(2);
    let h = g.add_card_to_hand(0, catalog::deep_forest_hermit());
    cast(&mut g, h, &[]);
    let sq = named(&g, 0, "Squirrel");
    assert_eq!(sq.len(), 4);
    assert!(sq.iter().all(|&s| pt(&g, s) == (2, 2)));
    assert_eq!(g.battlefield_find(h).unwrap().counter_count(CounterType::Time), 3);
}

/// Hazel of the Rootbloom: at your end step a Squirrel token is copied twice.
#[test]
fn hazel_doubles_a_squirrel_at_end_step() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::hazel_of_the_rootbloom());
    let s = g.add_card_to_hand(0, catalog::swarmyard_massacre());
    cast(&mut g, s, &[]);
    assert_eq!(named(&g, 0, "Squirrel").len(), 2);
    to_end_step(&mut g);
    assert_eq!(named(&g, 0, "Squirrel").len(), 4);
}

/// Hazel's mana ability taps the other untapped tokens for that much mana.
#[test]
fn hazel_taps_tokens_for_mana() {
    let mut g = pod(2);
    let hazel = g.add_card_to_battlefield(0, catalog::hazel_of_the_rootbloom());
    g.clear_sickness(hazel);
    let s = g.add_card_to_hand(0, catalog::swarmyard_massacre());
    cast(&mut g, s, &[]);
    g.players[0].mana_pool.empty();
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: hazel,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("mana");
    assert_eq!(g.players[0].mana_pool.total(), 2);
    assert_eq!(g.players[0].life, life - 2);
    assert!(named(&g, 0, "Squirrel").iter().all(|&s| g.battlefield_find(s).unwrap().tapped));
}

/// Saw in Half: two copies at half power and toughness, rounded up.
#[test]
fn saw_in_half_splits_a_wurm() {
    let mut g = pod(2);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let saw = g.add_card_to_hand(0, catalog::saw_in_half());
    cast(&mut g, saw, &[Target::Permanent(wurm)]);
    let halves = named(&g, 1, "Craw Wurm");
    assert_eq!(halves.len(), 2);
    assert!(halves.iter().all(|&h| pt(&g, h) == (3, 2)), "6/4 halves to 3/2");
}

/// Swarmyard Massacre: -1/-1 per vermin you control to everything else.
#[test]
fn swarmyard_massacre_spares_vermin() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let m = g.add_card_to_hand(0, catalog::swarmyard_massacre());
    cast(&mut g, m, &[]);
    assert!(g.battlefield_find(bear).is_none(), "2/2 takes -2/-2");
    assert_eq!(pt(&g, giant), (1, 1));
    assert_eq!(named(&g, 0, "Squirrel").len(), 2);
}

/// Sword of the Squeak counts your base-1 creatures.
#[test]
fn sword_of_the_squeak_counts_the_small() {
    let mut g = pod(2);
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_the_squeak());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::llanowar_elves());
    }
    g.add_card_to_battlefield(1, catalog::llanowar_elves());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: sword, target: bear }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (4, 4), "two 1/1 Elves of ours; theirs doesn't count");
}

/// Windgrace's Judgment: one nonland permanent per opponent.
#[test]
fn windgraces_judgment_takes_one_from_each() {
    let mut g = pod(3);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::sol_ring());
    let wj = g.add_card_to_hand(0, catalog::windgraces_judgment());
    cast(&mut g, wj, &[Target::Permanent(a), Target::Permanent(b)]);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    let a2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let a3 = g.add_card_to_battlefield(1, catalog::hill_giant());
    let wj2 = g.add_card_to_hand(0, catalog::windgraces_judgment());
    flood(&mut g, 0);
    let two_of_one = g.perform_action(GameAction::CastSpell {
        card_id: wj2,
        target: Some(Target::Permanent(a2)),
        additional_targets: vec![Target::Permanent(a3)],
        mode: None,
        x_value: None,
    });
    assert!(two_of_one.is_err(), "two targets that one opponent controls");
}

/// Gourmand's Talent: on your turn artifacts are Foods; level 2 makes a
/// Raccoon on the first life gain.
#[test]
fn gourmands_talent_feeds_and_levels() {
    let mut g = pod(2);
    let t = g.add_card_to_hand(0, catalog::gourmands_talent());
    cast(&mut g, t, &[]);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    assert!(g.computed_permanent(ring).unwrap().subtypes().artifact_subtypes.contains(&crabomination::card::ArtifactSubtype::Food));
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: t,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("level 2");
    drain_stack(&mut g);
    let salve = g.add_card_to_hand(0, catalog::healing_salve());
    cast(&mut g, salve, &[Target::Player(0)]);
    assert_eq!(named(&g, 0, "Raccoon").len(), 1);
    let salve2 = g.add_card_to_hand(0, catalog::healing_salve());
    cast(&mut g, salve2, &[Target::Player(0)]);
    assert_eq!(named(&g, 0, "Raccoon").len(), 1, "first gain each turn only");
}

/// Garruk's 0 makes two Wolves; a Wolf dying feeds Garruk a loyalty counter.
#[test]
fn garruks_wolves_feed_him() {
    let mut g = pod(2);
    let garruk = g.add_card_to_battlefield(0, catalog::garruk_cursed_huntsman());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: garruk, ability_index: 0, target: None, x_value: None })
        .expect("0");
    drain_stack(&mut g);
    let wolves = named(&g, 0, "Wolf");
    assert_eq!(wolves.len(), 2);
    let before = g.battlefield_find(garruk).unwrap().counter_count(CounterType::Loyalty);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(wolves[0])]);
    assert_eq!(g.battlefield_find(garruk).unwrap().counter_count(CounterType::Loyalty), before + 1);
}

/// Insatiable Frugivore eats the graveyard three at a time for Foods.
#[test]
fn insatiable_frugivore_turns_graveyard_into_food() {
    let mut g = pod(2);
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    let f = g.add_card_to_hand(0, catalog::insatiable_frugivore());
    cast(&mut g, f, &[]);
    assert_eq!(named(&g, 0, "Food").len(), 3, "one, then two more");
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Moonstone Eulogist: an opponent's creature dying makes you a Blood.
#[test]
fn moonstone_eulogist_bleeds_their_dead() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::moonstone_eulogist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(bear)]);
    assert_eq!(named(&g, 0, "Blood").len(), 1);
}

/// The Odd Acorn Gang: Squirrels connecting draw one card per batch.
#[test]
fn odd_acorn_gang_draws_when_squirrels_connect() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::the_odd_acorn_gang());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let s = g.add_card_to_hand(0, catalog::swarmyard_massacre());
    cast(&mut g, s, &[]);
    let sq = named(&g, 0, "Squirrel");
    for &s in &sq {
        g.clear_sickness(s);
    }
    let hand = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let atks = sq.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect();
    g.perform_action(GameAction::DeclareAttackers(atks)).expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1);
}
