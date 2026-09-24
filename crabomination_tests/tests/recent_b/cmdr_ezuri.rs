//! Commander: the Swell the Host precon (C15, Ezuri, `decks::cmdr_ezuri`).

use crabomination::card::{CardId, CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, g.priority.player_with_priority);
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on battlefield");
    (c.power, c.toughness)
}

/// Declare `attacks` for the active seat, run `blocks` (from `blocker`), and
/// step to the end of combat.
fn combat(g: &mut GameState, attacks: Vec<(CardId, usize)>, blocker: usize, blocks: Vec<(CardId, CardId)>) {
    for (a, _) in &attacks {
        g.clear_sickness(*a);
    }
    let active = g.active_player_idx;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = active;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = blocker;
    g.perform_action(GameAction::DeclareBlockers(blocks)).expect("blocks");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// CR 702.116a — a myriad copy carries the draw trigger too, so a Viper that
/// hits one opponent while its copy hits the other draws twice.
#[test]
fn broodbirth_viper_and_its_copy_each_draw() {
    let mut g = main_phase(3);
    let v = g.add_card_to_battlefield(0, catalog::broodbirth_viper());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    combat(&mut g, vec![(v, 1)], 1, vec![]);
    assert_eq!(g.players[1].life, 17);
    assert_eq!(g.players[2].life, 17, "the copy attacked the other opponent");
    assert_eq!(g.players[0].hand.len(), hand + 2);
    assert!(!g.battlefield.iter().any(|c| c.is_token), "the copy is exiled at end of combat");
}

/// X is the Colossus's power as the ability resolves, so a second activation
/// doubles the doubled body.
#[test]
fn chameleon_colossus_doubles_its_power() {
    let mut g = main_phase(2);
    let c = g.add_card_to_battlefield(0, catalog::chameleon_colossus());
    activate(&mut g, c, None);
    assert_eq!(pt(&g, c), (8, 8));
    activate(&mut g, c, None);
    assert_eq!(pt(&g, c), (16, 16));
}

#[test]
fn great_oak_guardian_untaps_and_pumps_the_targeted_players_team() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).expect("bear").tapped = true;
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let oak = g.add_card_to_hand(0, catalog::great_oak_guardian());
    cast(&mut g, oak, Some(Target::Player(0))).expect("cast");
    assert!(!g.battlefield_find(bear).expect("bear").tapped, "untapped");
    assert_eq!(pt(&g, bear), (4, 4));
    assert_eq!(pt(&g, theirs), (2, 2), "the other player's creatures are untouched");
}

#[test]
fn illusory_ambusher_draws_the_damage_it_is_dealt() {
    let mut g = main_phase(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let a = g.add_card_to_battlefield(0, catalog::illusory_ambusher());
    let hand = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(a))).expect("bolt");
    assert_eq!(g.players[0].hand.len(), hand + 3, "three damage, three cards (it died too)");
}

#[test]
fn kaseto_makes_a_snake_bigger_and_anything_unblockable() {
    let mut g = main_phase(2);
    let kaseto = g.add_card_to_battlefield(0, catalog::kaseto_orochi_archmage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, kaseto, Some(Target::Permanent(kaseto)));
    assert_eq!(pt(&g, kaseto), (4, 4), "a Snake gets +2/+2");
    activate(&mut g, kaseto, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (2, 2), "a Bear does not");
    let wall = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(wall, bear)])).is_err(), "can't be blocked");
}

/// CR 702.58a — graft on a land: it enters with its counter and hands it to
/// the next creature.
#[test]
fn llanowar_reborn_grafts_onto_an_entering_creature() {
    let mut g = main_phase(2);
    let land = g.add_card_to_hand(0, catalog::llanowar_reborn());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    let l = g.battlefield_find(land).expect("land");
    assert!(l.tapped);
    assert_eq!(l.counter_count(CounterType::PlusOnePlusOne), 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None).expect("bear");
    assert_eq!(pt(&g, bear), (3, 3));
    assert_eq!(g.battlefield_find(land).expect("land").counter_count(CounterType::PlusOnePlusOne), 0);
}

#[test]
fn lorescale_coatl_grows_on_each_draw() {
    let mut g = main_phase(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let c = g.add_card_to_battlefield(0, catalog::lorescale_coatl());
    let divination = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, divination, None).expect("draw two");
    assert_eq!(pt(&g, c), (4, 4));
}

/// Each copy blocks its original and is exiled at end of combat; outside
/// the declare blockers step the spell can't be cast.
#[test]
fn mirror_match_blocks_each_attacker_with_its_copy() {
    let mut g = main_phase(2);
    g.active_player_idx = 1;
    let mm = g.add_card_to_hand(0, catalog::mirror_match());
    g.priority.player_with_priority = 0;
    assert!(cast(&mut g, mm, None).is_err(), "not in a main phase");
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(0) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no declared blocks");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    cast(&mut g, mm, None).expect("cast in declare blockers");
    let copies: Vec<CardId> = g.battlefield.iter().filter(|c| c.is_token && c.controller == 0).map(|c| c.id).collect();
    assert_eq!(copies.len(), 2);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, 20, "both attackers were blocked");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "each traded with its twin");
    assert!(!g.battlefield.iter().any(|c| c.is_token), "no copy outlives combat");
}

/// The creature it damaged is destroyed at end of combat even though it
/// survived the damage.
#[test]
fn ohran_viper_destroys_what_it_fought_at_end_of_combat() {
    let mut g = main_phase(2);
    g.active_player_idx = 1;
    let giant = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let viper = g.add_card_to_battlefield(0, catalog::ohran_viper());
    combat(&mut g, vec![(giant, 0)], 0, vec![(viper, giant)]);
    assert!(g.battlefield_find(giant).is_none(), "destroyed at end of combat");
    assert!(g.battlefield_find(viper).is_some());
}

/// "Choose an opponent. That player returns a card" — the opponent's own
/// graveyard, at a table where the other opponent's graveyard is full too.
#[test]
fn skullwinder_returns_one_card_each_for_you_and_a_chosen_opponent() {
    let mut g = main_phase(3);
    let mine = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let one = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let two = g.add_card_to_graveyard(2, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::skullwinder());
    cast(&mut g, s, None).expect("cast");
    // The ETB's target is chosen by the auto-targeter: the only card in the
    // caster's graveyard.
    assert!(g.players[0].hand.iter().any(|c| c.id == mine));
    assert!(g.players[2].hand.iter().any(|c| c.id == two), "the opponent with fewer creatures got the gift");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == one));
}

#[test]
fn stingerfling_spider_shoots_down_a_flier() {
    let mut g = main_phase(2);
    let bird = g.add_card_to_battlefield(1, catalog::serra_angel());
    let s = g.add_card_to_hand(0, catalog::stingerfling_spider());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, s, None).expect("cast");
    assert!(g.battlefield_find(bird).is_none());
}

#[test]
fn synthetic_destiny_trades_the_board_for_as_many_new_creatures() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for def in [catalog::island(), catalog::hill_giant(), catalog::island()] {
        g.add_card_to_library(0, def);
    }
    let sd = g.add_card_to_hand(0, catalog::synthetic_destiny());
    cast(&mut g, sd, None).expect("cast");
    assert!(!g.battlefield.iter().any(|c| c.controller == 0 && c.definition.is_creature()));
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Hill Giant"));
    assert_eq!(g.players[0].library.len(), 2);
}

/// CR 708.8 — turning the Hermit face up mints four Saprolings, and its
/// static pumps every Saproling.
#[test]
fn thelonite_hermit_unmorphs_into_an_army() {
    let mut g = main_phase(2);
    let h = g.add_card_to_hand(0, catalog::thelonite_hermit());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFaceDown { card_id: h }).expect("face down");
    drain_stack(&mut g);
    flood(&mut g, 0);
    g.perform_action(GameAction::TurnFaceUp { card_id: h }).expect("face up");
    drain_stack(&mut g);
    let saps: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.definition.name == "Saproling").map(|c| c.id).collect();
    assert_eq!(saps.len(), 4);
    assert_eq!(pt(&g, saps[0]), (2, 2));
}

/// The default picks: two +1/+1 counters and two basic lands.
#[test]
fn verdant_confluence_default_modes() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let vc = g.add_card_to_hand(0, catalog::verdant_confluence());
    cast(&mut g, vc, Some(Target::Permanent(bear))).expect("cast");
    assert_eq!(pt(&g, bear), (4, 4));
    let forests = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Forest").count();
    assert_eq!(forests, 2);
}

/// CR 107.4e — each {G/U} pip is paid with either color.
#[test]
fn wistful_selkie_pays_hybrid_with_blue_alone() {
    let mut g = main_phase(2);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let s = g.add_card_to_hand(0, catalog::wistful_selkie());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.perform_action(GameAction::CastSpell { card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("three blue pays {G/U}x3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(s).is_some());
    assert_eq!(g.players[0].hand.len(), 1, "drew on entry");
}

/// CR 708.2 / 702.37 — a land with morph is cast face down as a 2/2 creature
/// and turned face up it is a land again.
#[test]
fn zoetic_cavern_hides_as_a_creature_and_flips_back_to_a_land() {
    let mut g = main_phase(2);
    let z = g.add_card_to_hand(0, catalog::zoetic_cavern());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFaceDown { card_id: z }).expect("face down");
    drain_stack(&mut g);
    assert_eq!(pt(&g, z), (2, 2));
    flood(&mut g, 0);
    g.perform_action(GameAction::TurnFaceUp { card_id: z }).expect("face up");
    drain_stack(&mut g);
    let c = g.computed_permanent(z).expect("still here");
    let types = c.card_types();
    assert!(types.contains(&CardType::Land) && !types.contains(&CardType::Creature));
}
