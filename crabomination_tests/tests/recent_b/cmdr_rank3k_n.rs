//! Commander: two- and one-card gaps of EDHREC ranks 2501-3500
//! (`decks::cmdr_rank3k_n`).

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
    g
}

fn flood(g: &mut GameState) {
    for c in [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    );
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    );
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne)
}

fn kill(g: &mut GameState, id: CardId) {
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(g, murder, Some(Target::Permanent(id)));
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks
            .into_iter()
            .map(|(attacker, p)| Attack {
                attacker,
                target: AttackTarget::Player(p),
            })
            .collect(),
    ))
    .expect("attack");
    drain_stack(g);
}
/// CR 605.1a — Daily Bugle Building taps for {C} or filtered colour, and
/// Smear Campaign gives a legend menace.
#[test]
fn daily_bugle_building_smears_a_legend() {
    let mut g = pod(2);
    let bugle = ready(&mut g, 0, catalog::daily_bugle_building());
    let mj = ready(&mut g, 0, catalog::mj_rising_star());
    activate(&mut g, bugle, 2, Some(Target::Permanent(mj)));
    assert!(
        g.computed_permanent(mj)
            .unwrap()
            .keywords()
            .contains(&Keyword::Menace)
    );
}

/// CR 509.1h — Labyrinth Raptor makes the defender sacrifice a blocker of a
/// menace attacker.
#[test]
fn labyrinth_raptor_eats_a_blocker() {
    let mut g = pod(2);
    let raptor = ready(&mut g, 0, catalog::labyrinth_raptor());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    declare(&mut g, vec![(raptor, 1)]);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(a, raptor), (b, raptor)]))
        .expect("double block");
    drain_stack(&mut g);
    let left = [a, b]
        .iter()
        .filter(|id| g.battlefield_find(**id).is_some())
        .count();
    assert_eq!(left, 1);
    assert_eq!(g.players[1].graveyard.len(), 1);
}

/// CR 106.1 — Fogwell's Gym pings for red and loots for a Spider counter.
#[test]
fn fogwells_gym_feeds_lady_spider() {
    let mut g = pod(2);
    let gym = ready(&mut g, 0, catalog::fogwells_gym());
    let lady = ready(&mut g, 0, catalog::lady_spider_maybelle_reilly());
    g.players[0].mana_pool.empty();
    g.perform_action(GameAction::ActivateAbility {
        card_id: gym,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap");
    assert_eq!(g.players[0].life, 19);
    g.battlefield_find_mut(gym).unwrap().tapped = false;
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    activate(&mut g, gym, 1, None);
    assert_eq!(plus(&g, lady), 1, "the discard grew the Spider");
    assert_eq!(g.players[0].hand.len(), 1);
}

/// CR 707.2 — Cemetery Puca becomes a copy of a dead creature, keeping its
/// trigger.
#[test]
fn cemetery_puca_copies_the_dead() {
    let mut g = pod(2);
    let puca = ready(&mut g, 0, catalog::cemetery_puca());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    kill(&mut g, angel);
    assert_eq!(g.computed_permanent(puca).unwrap().def.name, "Serra Angel");
    assert_eq!(pt(&g, puca), (4, 4));
    kill(&mut g, bear);
    assert_eq!(
        g.computed_permanent(puca).unwrap().def.name,
        "Grizzly Bears",
        "it kept the copy trigger"
    );
}
