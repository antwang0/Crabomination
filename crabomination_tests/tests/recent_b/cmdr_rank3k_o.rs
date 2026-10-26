//! Commander: one- and two-card gaps of EDHREC ranks 2501-3500
//! (`decks::cmdr_rank3k_o`).

use crabomination::card::{CardId, CounterType, CreatureType};
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
/// CR 707.10 — Rimefire Torque banks charge counters off its chosen type and
/// spends three on a copy of the next instant.
#[test]
fn rimefire_torque_charges_and_copies() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(
        CreatureType::Bear,
    )]));
    let torque = g.add_card_to_hand(0, catalog::rimefire_torque());
    cast(&mut g, torque, None);
    g.clear_sickness(torque);
    for _ in 0..3 {
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast(&mut g, bear, None);
    }
    assert_eq!(
        g.battlefield_find(torque)
            .unwrap()
            .counter_count(CounterType::Charge),
        3
    );
    activate(&mut g, torque, 0, None);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 14, "bolt and its copy");
}

/// CR 601.2f — Gathering Stone discounts its type and digs on entry; taking
/// the card is a "may" (declining both leaves it on top).
#[test]
fn gathering_stone_digs_its_type() {
    for take in [true, false] {
        let mut g = pod(2);
        let bear = g.add_card_to_library(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::CreatureType(CreatureType::Bear),
            DecisionAnswer::Bool(take),
            DecisionAnswer::Bool(false),
        ]));
        let stone = g.add_card_to_hand(0, catalog::gathering_stone());
        cast(&mut g, stone, None);
        assert_eq!(g.players[0].hand.len(), usize::from(take), "the Bear came to hand: {take}");
        assert_eq!(g.players[0].library.iter().any(|c| c.id == bear), !take);
    }
}

/// CR 506.5 — Eiganjo Exemplar pumps a Samurai attacking alone.
#[test]
fn eiganjo_exemplar_pumps_a_lone_samurai() {
    let mut g = pod(2);
    let ex = ready(&mut g, 0, catalog::eiganjo_exemplar());
    declare(&mut g, vec![(ex, 1)]);
    assert_eq!(pt(&g, ex), (3, 2));
}

/// CR 702.193 — Loki copies a cheap enough instant; power-up grows him.
#[test]
fn loki_copies_cheap_spells() {
    let mut g = pod(2);
    let loki = ready(&mut g, 0, catalog::loki_laufeyson());
    activate(&mut g, loki, 0, None);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 14);
    activate(&mut g, loki, 1, None);
    assert_eq!(plus(&g, loki), 2);
}

/// CR 603.7c — Loki's watch is for the next instant or sorcery with mana value
/// at most his power: a pricier one (Lava Axe, 5 > 2) leaves it armed, and the
/// Bolt after it is copied.
#[test]
fn loki_skips_a_spell_above_his_power_and_copies_the_next() {
    let mut g = pod(2);
    let loki = ready(&mut g, 0, catalog::loki_laufeyson());
    activate(&mut g, loki, 0, None);
    let axe = g.add_card_to_hand(0, catalog::lava_axe());
    cast(&mut g, axe, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 15, "Lava Axe, not copied");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 9, "Bolt and its copy");
}

/// CR 603.2 — Wanda's Vision fires on the second spell each turn.
#[test]
fn wandas_vision_cascades_on_the_second_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::wandas_vision());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    for n in 0..2 {
        let opt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, opt, Some(Target::Player(1)));
        // Exiled — and cast for free as it resolves — only off the second.
        let left_library = !g.players[0].library.iter().any(|c| c.definition.name == "Grizzly Bears");
        assert_eq!(left_library, n == 1);
    }
}

/// Wanda's Vision's free cast happens as the trigger resolves (CR 608.2g):
/// taken, the exiled Bears is on the battlefield before anything else.
#[test]
fn wandas_vision_casts_the_exiled_card_as_it_resolves() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::wandas_vision());
    g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..2 {
        let opt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        cast(&mut g, opt, Some(Target::Player(1)));
    }
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0));
}

/// CR 119.3 — Sengir grows on deaths and drinks a departed player's life.
#[test]
fn sengir_drinks_a_losers_life() {
    let mut g = pod(3);
    let sengir = ready(&mut g, 0, catalog::sengir_the_dark_baron());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    kill(&mut g, bear);
    assert_eq!(plus(&g, sengir), 2);
    g.players[2].life_at_turn_start = 7;
    g.players[2].life = 0;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 27);
}
