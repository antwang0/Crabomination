//! Commander: Raphael's, Mishra Claimed by Gix's and Tom, Bert, and William's
//! last gaps (`decks::cmdr_edhrec42`).

use crabomination::card::{CardId, CardType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// Zariel: +1 is a team pump with haste, 0 a Devil that pings as it dies,
/// −6 an emblem whose first-combat trigger banks another combat (CR 500.8).
#[test]
fn cr_500_8_zariel_devils_and_extra_combat() {
    let mut g = pod(2);
    let zariel = g.add_card_to_battlefield(0, catalog::zariel_archduke_of_avernus());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    loyalty(&mut g, zariel, 0, None).expect("+1");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3);
    assert!(cp.keywords().contains(&Keyword::Haste));
    g.turn_number += 1;
    g.battlefield_find_mut(zariel).unwrap().loyalty_uses_this_turn = 0;
    loyalty(&mut g, zariel, 1, None).expect("0");
    let devil = named(&g, 0, "Devil");
    assert_eq!(devil.len(), 1);
    let life = g.players[1].life;
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Permanent(devil[0]))).expect("shock the devil");
    assert_eq!(g.players[1].life, life - 1, "the Devil's death ping");

    g.battlefield_find_mut(zariel).unwrap().counters.insert(crabomination::card::CounterType::Loyalty, 6);
    g.battlefield_find_mut(zariel).unwrap().loyalty_uses_this_turn = 0;
    loyalty(&mut g, zariel, 2, None).expect("-6");
    g.step = TurnStep::EndCombat;
    g.fire_step_triggers(TurnStep::EndCombat);
    drain_stack(&mut g);
    assert_eq!(g.additional_combat_phases, 1);
}

/// CR 701.42 — Hanweir Battlements melds with Hanweir Garrison into Hanweir,
/// the Writhing Township, whose attack makes two attacking Eldrazi Horrors.
#[test]
fn cr_701_42_hanweir_melds() {
    let mut g = pod(2);
    let battlements = g.add_card_to_battlefield(0, catalog::hanweir_battlements());
    g.add_card_to_battlefield(0, catalog::hanweir_garrison());
    flood(&mut g);
    activate(&mut g, battlements, 2, None).expect("meld");
    let township = named(&g, 0, "Hanweir, the Writhing Township");
    assert_eq!(township.len(), 1);
    assert!(named(&g, 0, "Hanweir Garrison").is_empty());
    g.clear_sickness(township[0]);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: township[0],
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Eldrazi Horror").len(), 2);
}

/// Tilonalli's Summoner: {X}{R} on attack makes X attacking Elementals,
/// exiled at the next end step without the city's blessing (CR 702.131).
#[test]
fn cr_702_131_tilonallis_summoner_elementals_leave_without_the_blessing() {
    let mut g = pod(2);
    let summoner = g.add_card_to_battlefield(0, catalog::tilonallis_summoner());
    g.clear_sickness(summoner);
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: summoner, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Elemental").len(), 2);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(named(&g, 0, "Elemental").is_empty(), "no blessing: exiled");
}

/// Tom, Bert, and William trade a creature for cards, and come back from
/// dying as a noncreature artifact.
#[test]
fn tom_bert_and_william_turn_to_stone() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let trolls = g.add_card_to_battlefield(0, catalog::tom_bert_and_william());
    g.add_card_to_battlefield(0, catalog::hill_giant());
    flood(&mut g);
    let hand = g.players[0].hand.len();
    activate(&mut g, trolls, 0, None).expect("sacrifice the giant");
    assert_eq!(g.players[0].hand.len(), hand + 3 - 1, "draw three, discard one");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.battlefield_find_mut(trolls).unwrap().damage = 3;
    cast(&mut g, bolt, Some(Target::Permanent(trolls))).expect("bolt");
    let back = named(&g, 0, "Tom, Bert, and William");
    assert_eq!(back.len(), 1, "returned");
    let cp = g.computed_permanent(back[0]).unwrap();
    assert!(cp.card_types().contains(&CardType::Artifact));
    assert!(!cp.card_types().contains(&CardType::Creature));
}

/// Bebop & Rocksteady attacking: discard a card, or (empty-handed) sacrifice
/// a permanent.
#[test]
fn bebop_rocksteady_tax_every_swing() {
    let mut g = pod(2);
    let duo = g.add_card_to_battlefield(0, catalog::bebop_rocksteady());
    g.clear_sickness(duo);
    g.players[0].hand.clear();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: duo, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let gone = g.battlefield_find(land).is_none() || g.battlefield_find(duo).is_none();
    assert!(gone, "a permanent was sacrificed");
}

/// Part in Friendship: a nontoken creature dying reveals a creature card — onto
/// the battlefield when its mana value fits your land count, else to hand —
/// once each turn.
#[test]
fn part_in_friendship_replaces_the_fallen() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::part_in_friendship());
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::hill_giant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Permanent(bear))).expect("shock");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1, "a two-drop with two lands: onto the battlefield");
    let second = named(&g, 0, "Grizzly Bears")[0];
    let hand = g.players[0].hand.len();
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Permanent(second))).expect("shock again");
    assert_eq!(g.players[0].hand.len(), hand, "once each turn: the Giant stays put");
}
