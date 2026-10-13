//! Commander: Tsagan's and The Balrog's last gaps (`decks::cmdr_edhrec43`).

use crabomination::card::{CardId, Keyword};
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

fn give(g: &mut GameState, each: u32) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, each);
    }
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    give(g, 20);
    g.players[0].mana_pool.add_colorless(20);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// CR 702.179 — Tsagan pumps per first/double striker on attack; at max speed
/// it has deathtouch and the team has first strike.
#[test]
fn cr_702_179_tsagan_strikes_first_at_max_speed() {
    let mut g = pod(2);
    let tsagan = ready(&mut g, 0, catalog::tsagan_raider_warlord());
    let sentry = ready(&mut g, 0, catalog::kinscaer_sentry());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::FirstStrike));
    g.players[0].hand.clear();
    attack(&mut g, &[tsagan, sentry, bear]);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2 + 2, "Tsagan and the Sentry strike first");
    g.players[0].speed = 4;
    assert!(g.computed_permanent(tsagan).unwrap().keywords().contains(&Keyword::Deathtouch));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::FirstStrike));
}

/// Kinscaer Sentry deploys a creature card with mana value up to its
/// attacker count, tapped and attacking (CR 508.4).
#[test]
fn cr_508_4_kinscaer_sentry_deploys_by_attacker_count() {
    let mut g = pod(2);
    let sentry = ready(&mut g, 0, catalog::kinscaer_sentry());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].hand.clear();
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    let elves = g.add_card_to_hand(0, catalog::llanowar_elves());
    attack(&mut g, &[sentry, bear]);
    let e = g.battlefield_find(elves).expect("the one-drop deployed");
    assert!(e.tapped);
    assert!(g.players[0].hand.iter().any(|c| c.id == giant), "a four-drop doesn't fit two attackers");
}

/// Smellerbee: a discarded hand becomes one card per attacker.
#[test]
fn smellerbee_trades_the_hand_for_attackers() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let bee = ready(&mut g, 0, catalog::smellerbee_rebel_fighter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Haste));
    g.players[0].hand.clear();
    g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, &[bee, bear]);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Lobelia mints Treasures off a creature an opponent lost this turn.
#[test]
fn lobelia_cashes_in_a_fresh_corpse() {
    let mut g = pod(3);
    let giant = g.add_card_to_battlefield(2, catalog::hill_giant());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(giant))).expect("bolt");
    let lobelia = g.add_card_to_hand(0, catalog::lobelia_sackville_baggins());
    cast(&mut g, lobelia, None).expect("lobelia");
    assert!(g.exile.iter().any(|c| c.id == giant));
    assert_eq!(named(&g, 0, "Treasure").len(), 3);
}

/// Lash of the Balrog's additional cost: with no creature to sacrifice, {4}.
#[test]
fn lash_of_the_balrog_pays_four_without_fodder() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let lash = g.add_card_to_hand(0, catalog::lash_of_the_balrog());
    give(&mut g, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: lash,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("lash");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// CR 903.3 — Visions of Ruin's flashback is cheaper by your commander's
/// mana value: a five-drop commander in the command zone takes {8}{R}{R}
/// to {3}{R}{R}.
#[test]
fn cr_903_3_visions_of_ruin_flashback_reads_the_commander() {
    let mut g = pod(3);
    g.seat_commanders(0, vec![catalog::tom_bert_and_william()]);
    for seat in 1..3 {
        g.add_card_to_battlefield(seat, catalog::sol_ring());
    }
    let visions = g.add_card_to_graveyard(0, catalog::visions_of_ruin());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFlashback {
        card_id: visions,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("five mana is enough");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Treasure").len(), 2, "one per opponent's sacrificed artifact");
    assert!(g.exile.iter().any(|c| c.id == visions));
}
