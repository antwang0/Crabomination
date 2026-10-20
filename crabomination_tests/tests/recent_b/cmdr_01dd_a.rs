//! Commander: the 2026-10-20 (`01DDeUtc`) closers (`decks::cmdr_01dd_*`).

use crabomination::card::{CardId, CardType, CreatureType, Subtypes, TokenDefinition};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn beast(power: i32) -> TokenDefinition {
    TokenDefinition {
        name: "Beast".into(),
        power,
        toughness: power,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        ..Default::default()
    }
}

fn treasures(g: &GameState, seat: usize) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == "Treasure").count()
}

/// CR 702.110a-b — Henry Wu grants exploit to a Human entering; exploiting a
/// non-Human with power 3 draws a card and makes a Treasure.
#[test]
fn cr_702_110b_henry_wu_pays_for_a_big_non_human() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::henry_wu_ingen_geneticist());
    let token = g.add_token_to_battlefield(0, &beast(3));
    let vanguard = g.add_card_to_hand(0, catalog::elite_vanguard());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    cast(&mut g, vanguard, None).expect("cast");
    assert!(g.battlefield_find(token).is_none(), "the Beast was exploited");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    assert_eq!(treasures(&g, 0), 1, "power 3 or greater");
}

/// CR 702.110b — a non-Human with power under 3 draws without a Treasure.
#[test]
fn henry_wu_small_non_human_draws_only() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::henry_wu_ingen_geneticist());
    g.add_token_to_battlefield(0, &beast(2));
    let vanguard = g.add_card_to_hand(0, catalog::elite_vanguard());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    cast(&mut g, vanguard, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand);
    assert_eq!(treasures(&g, 0), 0);
}

/// CR 702.110b — Skull Skaab's Zombie needs a nontoken exploit: a token
/// exploited makes nothing, a Grizzly Bears exploited makes a 2/2 Zombie.
#[test]
fn cr_702_110b_skull_skaab_needs_a_nontoken_creature() {
    let zombies = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Zombie" && c.is_token).count();
    let mut g = pod(2);
    g.add_token_to_battlefield(0, &beast(1));
    let skaab = g.add_card_to_hand(0, catalog::skull_skaab());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, skaab, None).expect("cast");
    assert_eq!(zombies(&g), 0, "a token exploited");

    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let skaab = g.add_card_to_hand(0, catalog::skull_skaab());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, skaab, None).expect("cast");
    assert!(g.battlefield_find(bears).is_none());
    assert_eq!(zombies(&g), 1, "a nontoken creature exploited");
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn attack(g: &mut GameState, attacker: CardId, defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker,
        target: crabomination::game::types::AttackTarget::Player(defender),
    }]))
    .expect("attack");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

/// Harpoon Sniper's X counts every Merfolk you control, itself included:
/// with two others it deals 3 to a blocker-less attacker.
#[test]
fn harpoon_sniper_counts_your_merfolk() {
    let mut g = pod(2);
    let sniper = ready(&mut g, 1, catalog::harpoon_sniper());
    ready(&mut g, 1, catalog::judge_of_currents());
    ready(&mut g, 1, catalog::harpoon_sniper());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: giant,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.players[1].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sniper,
        ability_index: 0,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none(), "3 damage to a 3/3");
}

/// Judge of Currents fires for a Merfolk of yours becoming tapped (an attack
/// taps it) and not for an opponent's.
#[test]
fn judge_of_currents_gains_on_your_merfolk_tapping() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::judge_of_currents());
    let sniper = ready(&mut g, 0, catalog::harpoon_sniper());
    let theirs = ready(&mut g, 1, catalog::harpoon_sniper());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[0].life;
    attack(&mut g, sniper, 1);
    assert_eq!(g.players[0].life, life + 1);
    g.battlefield_find_mut(theirs).unwrap().tapped = false;
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: theirs, actor: None, as_attacker: false }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "an opponent's Merfolk");
}

/// Sygg, Wanderwine Wisdom's entry grants "deals combat damage to a player,
/// draw a card" to its target; Sygg can't be blocked and draws on the hit.
#[test]
fn sygg_wanderwine_wisdom_grants_a_draw_on_combat_damage() {
    let mut g = pod(2);
    let sygg = g.add_card_to_hand(0, catalog::sygg_wanderwine_wisdom());
    flood(&mut g, 0);
    cast(&mut g, sygg, None).expect("cast");
    g.clear_sickness(sygg);
    let hand = g.players[0].hand.len();
    attack(&mut g, sygg, 1);
    assert_eq!(g.players[1].life, 18, "unblocked");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew off the granted trigger");
}

/// Sygg transforms at your first main phase when you pay {W}; entering the
/// back face gives a creature you control protection from each color until
/// your next turn.
#[test]
fn sygg_pays_w_to_transform_and_shield() {
    let mut g = pod(2);
    let sygg = ready(&mut g, 0, catalog::sygg_wanderwine_wisdom());
    ready(&mut g, 0, catalog::plains());
    g.step = TurnStep::Upkeep;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sygg).unwrap().definition.name, "Sygg, Wanderbrine Shield");
    assert!(g.computed_permanent(sygg).unwrap().keywords().contains(&crabomination::card::Keyword::Protection(Color::Red)));
}
