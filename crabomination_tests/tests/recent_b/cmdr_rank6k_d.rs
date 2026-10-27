//! Commander: two-card gaps of EDHREC ranks 5501-6500
//! (`decks::cmdr_rank6k_d`).

use crabomination::card::CardId;
use crabomination::catalog;
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

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .count()
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

fn finish_combat(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![]))
        .expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}
/// CR 614.1a — with Eligeth out, Siani's attack scry becomes a draw.
#[test]
fn eligeth_turns_sianis_scry_into_draws() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::eligeth_crossroads_augur());
    let siani = ready(&mut g, 0, catalog::siani_eye_of_the_storm());
    let angel = ready(&mut g, 0, catalog::serra_angel());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    declare(&mut g, vec![(siani, 1), (angel, 1)]);
    assert_eq!(
        g.players[0].hand.len(),
        2,
        "two flying attackers: scry 2 became draw 2"
    );
}

/// CR 701.22a — without Eligeth, Siani scries.
#[test]
fn siani_scries_without_eligeth() {
    let mut g = pod(2);
    let siani = ready(&mut g, 0, catalog::siani_eye_of_the_storm());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    declare(&mut g, vec![(siani, 1)]);
    assert_eq!(g.players[0].hand.len(), 0);
}

/// CR 510.1 — Black Widow's hit exiles the defender's next spell for her
/// controller to cast.
#[test]
fn black_widow_steals_a_spell() {
    let mut g = pod(2);
    let bw = ready(&mut g, 0, catalog::black_widow_super_spy());
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::forest());
    declare(&mut g, vec![(bw, 1)]);
    finish_combat(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert!(g.exile.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Black Widow: the +1/+1 counter and the cast are exclusive — "you may put a
/// +1/+1 counter on Black Widow. If you don't, you may cast the exiled card".
#[test]
fn black_widow_counter_or_cast_not_both() {
    for take_counter in [true, false] {
        let mut g = pod(2);
        let bw = ready(&mut g, 0, catalog::black_widow_super_spy());
        g.add_card_to_library(1, catalog::grizzly_bears());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(take_counter),
        ]));
        declare(&mut g, vec![(bw, 1)]);
        finish_combat(&mut g);
        let counters = g.battlefield_find(bw).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne);
        let castable = g.exile.iter().any(|c| c.definition.name == "Grizzly Bears" && c.may_play_until.is_some());
        assert_eq!(counters == 1, take_counter);
        assert_eq!(castable, !take_counter, "the cast is open only without the counter");
    }
}

/// CR 702.178 — Petty Larceny exiles an opponent's top two and makes a
/// Treasure.
#[test]
fn petty_larceny_steals_two() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let pl = g.add_card_to_hand(0, catalog::petty_larceny());
    cast(&mut g, pl, Some(Target::Player(1)));
    assert_eq!(
        g.exile
            .iter()
            .filter(|c| c.definition.name == "Grizzly Bears")
            .count(),
        2
    );
    assert_eq!(named(&g, 0, "Treasure"), 1);
    // CR 406.3 — exiled face down, and still castable through the grant.
    let stolen: Vec<_> = g.exile.iter().filter(|c| c.definition.name == "Grizzly Bears").map(|c| (c.id, c.face_down)).collect();
    assert!(stolen.iter().all(|&(_, down)| down), "face down");
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: stolen[0].0, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast a face-down stolen card");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(stolen[0].0).map(|c| c.controller), Some(0));
}
