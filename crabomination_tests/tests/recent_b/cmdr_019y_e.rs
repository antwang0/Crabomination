//! Commander: the 2026-10-17 (`019Y6J4R`) fifth closer batch
//! (`decks::cmdr_019y_e`).

use crabomination::card::{CardId, CounterType, Keyword};
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
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(6);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    cast_mode(g, id, target, None)
}

fn cast_mode(
    g: &mut GameState,
    id: CardId,
    target: Option<Target>,
    mode: Option<usize>,
) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode,
        x_value: None,
    })?;
    drain_stack(g);
    Ok(())
}










fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}


/// Arc Blade deals 2 and re-suspends itself with three time counters.
#[test]
fn arc_blade_resuspends_itself() {
    let mut g = pod(2);
    let ab = g.add_card_to_hand(0, catalog::arc_blade());
    flood(&mut g, 0);
    let life = g.players[1].life;
    cast(&mut g, ab, Some(Target::Player(1))).expect("cast");
    assert_eq!(g.players[1].life, life - 2);
    let c = g.exile.iter().find(|c| c.id == ab).expect("exiled");
    assert_eq!(c.counter_count(CounterType::Time), 3);
}

/// Find the Path ventures and its land taps for {G}{G}.
#[test]
fn find_the_path_ventures_and_doubles_its_land() {
    let mut g = pod(2);
    let forest = ready(&mut g, 0, catalog::forest());
    let ftp = g.add_card_to_hand(0, catalog::find_the_path());
    flood(&mut g, 0);
    cast(&mut g, ftp, Some(Target::Permanent(forest))).expect("cast");
    assert!(g.players[0].dungeon.is_some());
    g.players[0].mana_pool = Default::default();
    let idx = g.battlefield_find(forest).unwrap().definition.activated_abilities.len();
    let ok = (0..=idx).any(|i| {
        let mut h = g.clone();
        activate(&mut h, forest, i, None).is_ok() && h.players[0].mana_pool.total() == 2
    });
    assert!(ok, "some ability of the land adds two mana");
}

/// Intrepid Outlander ventures with 6+ power attacking, not alone.
#[test]
fn intrepid_outlander_needs_a_pack() {
    let mut g = pod(2);
    let io = ready(&mut g, 0, catalog::intrepid_outlander());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: io, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.players[0].dungeon.is_none(), "2 power isn't 6");
    let mut g = pod(2);
    let io = ready(&mut g, 0, catalog::intrepid_outlander());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let lions = ready(&mut g, 0, catalog::savannah_lions());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        [io, giant, lions].map(|attacker| Attack { attacker, target: AttackTarget::Player(1) }).to_vec(),
    ))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.players[0].dungeon.is_some(), "2 + 3 + 2 = 7");
}

/// Anchovy & Banana Pizza kills a creature entering.
#[test]
fn anchovy_and_banana_pizza_destroys_on_entry() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let p = g.add_card_to_hand(0, catalog::anchovy_and_banana_pizza());
    flood(&mut g, 0);
    cast(&mut g, p, None).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
}

/// Tiger-Tribe Hunter flings a sacrificed creature's power at a creature.
#[test]
fn tiger_tribe_hunter_flings_its_pack() {
    let mut g = pod(2);
    let tth = ready(&mut g, 0, catalog::tiger_tribe_hunter());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let ogre = ready(&mut g, 1, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        [tth, giant].map(|attacker| Attack { attacker, target: AttackTarget::Player(1) }).to_vec(),
    ))
    .expect("attack with 7 power");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none(), "the Giant was sacrificed");
    assert!(g.battlefield_find(ogre).is_none(), "its 3 power killed the 3/3");
}

/// Disappearing Act counters a spell by returning a permanent you control.
#[test]
fn disappearing_act_bounces_one_of_yours_to_counter() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    let da = g.add_card_to_hand(0, catalog::disappearing_act());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: da,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "Bolt countered");
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "the Bears went home");
}

/// Abuelo's Awakening returns an artifact as a 1/1 flying Spirit plus X.
#[test]
fn abuelos_awakening_raises_an_artifact_spirit() {
    let mut g = pod(2);
    let rock = g.add_card_to_graveyard(0, catalog::sol_ring());
    let aa = g.add_card_to_hand(0, catalog::abuelos_awakening());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: aa,
        target: Some(Target::Permanent(rock)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(rock).expect("returned");
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords().contains(&Keyword::Flying));
}

/// Brilliance Unleashed's second mode makes a noncreature artifact a 3/3
/// flying Robot.
#[test]
fn brilliance_unleashed_returns_a_robot() {
    let mut g = pod(2);
    let rock = g.add_card_to_graveyard(0, catalog::sol_ring());
    let bu = g.add_card_to_hand(0, catalog::brilliance_unleashed());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bu,
        target: Some(Target::Permanent(rock)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(rock).expect("returned");
    assert_eq!((cp.power, cp.toughness), (3, 3));
}
