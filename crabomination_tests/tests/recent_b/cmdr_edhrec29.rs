//! Commander: Ojer Pakpatiq's, Bruna's, Maralen's and Gilgamesh's last gaps
//! (`decks::cmdr_edhrec29`).

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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, mode: Option<usize>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: None })
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

/// Sword Coast Serpent is unblockable only after a noncreature spell this
/// turn (CR 509.1b evasion via a conditional static).
#[test]
fn sword_coast_serpent_unblockable_after_noncreature_spell() {
    let mut g = pod(2);
    let serpent = ready(&mut g, 0, catalog::sword_coast_serpent());
    let kw = |g: &GameState| g.computed_permanent(serpent).unwrap().keywords().contains(&Keyword::Unblockable);
    assert!(!kw(&g));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)), None).expect("bolt");
    assert!(kw(&g));
}

/// Supreme Will's second mode digs four deep for one card.
#[test]
fn supreme_will_digs() {
    let mut g = pod(2);
    for def in [catalog::island, catalog::grizzly_bears, catalog::forest, catalog::mountain] {
        g.add_card_to_library(0, def());
    }
    let id = g.add_card_to_hand(0, catalog::supreme_will());
    cast(&mut g, id, None, Some(1)).expect("cast");
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].library.len(), 3);
}

/// Sovereigns of Lost Alara fetches an Aura onto a lone attacker.
#[test]
fn sovereigns_fetch_an_aura_for_the_lone_attacker() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sovereigns_of_lost_alara());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::curators_ward());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, &[bear]);
    let ward = g.battlefield.iter().find(|c| c.definition.name == "Curator's Ward").expect("fetched");
    assert_eq!(ward.attached_to, Some(bear));
}

/// Maralen stops draws and tutors at each draw step for 3 life.
#[test]
fn maralen_replaces_draws_with_tutors() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::maralen_of_the_mornsong());
    g.add_card_to_library(0, catalog::island());
    g.fire_step_triggers(TurnStep::Draw);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 17);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Mornsong Aria also stops life gain.
#[test]
fn mornsong_aria_stops_life_gain() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::mornsong_aria());
    let id = g.add_card_to_hand(0, catalog::healing_salve());
    let _ = cast(&mut g, id, Some(Target::Player(0)), None);
    assert_eq!(g.players[0].life, 20);
}

/// Goro-Goro's {R} gives your creatures haste.
#[test]
fn goro_goro_grants_haste() {
    let mut g = pod(2);
    let goro = ready(&mut g, 0, catalog::goro_goro_disciple_of_ryusei());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: goro,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("haste");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Haste));
}

/// Heiko attacking alone lets you cast an artifact from your graveyard.
#[test]
fn heiko_recasts_artifacts() {
    let mut g = pod(2);
    let heiko = ready(&mut g, 0, catalog::heiko_yamazaki_the_general());
    let sol = g.add_card_to_graveyard(0, catalog::sol_ring());
    attack(&mut g, &[heiko]);
    g.step = TurnStep::PostCombatMain;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: sol,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sol).is_some());
}
