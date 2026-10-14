//! Commander: two-card gaps of EDHREC ranks 2501-3500
//! (`decks::cmdr_rank3k_p`).

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
/// CR 702.124i — the two Ellies are Partner—Survivors; Brick Master hands an
/// attacker a Cordyceps Infected attacking the same opponent.
#[test]
fn ellie_brick_master_spawns_infected_attackers() {
    let mut g = pod(3);
    assert!(
        catalog::ellie_vengeful_hunter()
            .keywords
            .contains(&Keyword::PartnerLabel("Survivors".into()))
    );
    ready(&mut g, 0, catalog::ellie_brick_master());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(bear, 1)]);
    assert_eq!(named(&g, 0, "Cordyceps Infected"), 1);
    finish_combat(&mut g);
    assert_eq!(g.players[1].life, 17, "bear and the Infected");
}

/// CR 118.3 — Ellie, Vengeful Hunter pays life and a creature for 2 damage.
#[test]
fn ellie_vengeful_hunter_sacrifices_for_damage() {
    let mut g = pod(2);
    let ellie = ready(&mut g, 0, catalog::ellie_vengeful_hunter());
    ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, ellie, 0, Some(Target::Player(1)));
    assert_eq!(g.players[0].life, 18);
    assert_eq!(g.players[1].life, 18);
    assert!(
        g.computed_permanent(ellie)
            .unwrap()
            .keywords()
            .contains(&Keyword::Indestructible)
    );
}

/// CR 614.12 — Theorist's Sanctum enters tapped without a Jace, then empowers.
#[test]
fn theorists_sanctum_empowers_jace() {
    let mut g = pod(2);
    let land = g.add_card_to_hand(0, catalog::theorists_sanctum());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    assert!(
        g.battlefield_find(land).unwrap().tapped,
        "no Jace to behold"
    );
    g.battlefield_find_mut(land).unwrap().tapped = false;
    activate(&mut g, land, 1, None);
    let jace = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.name.contains("Jace"))
        .expect("Jace token");
    assert_eq!(jace.counter_count(CounterType::Loyalty), 2);
}

/// CR 111.10a — Hot Dog Cart makes a Food; Stardew Valley taps a creature for
/// another and can hand a permanent to an opponent.
#[test]
fn stardew_valley_cooks_and_gifts() {
    let mut g = pod(2);
    let cart = g.add_card_to_hand(0, catalog::hot_dog_cart());
    cast(&mut g, cart, None);
    assert_eq!(named(&g, 0, "Food"), 1);
    let valley = ready(&mut g, 0, catalog::stardew_valley());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, valley, 1, None);
    assert_eq!(named(&g, 0, "Food"), 2);
    assert!(g.battlefield_find(bear).unwrap().tapped);
    g.battlefield_find_mut(valley).unwrap().tapped = false;
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    activate(&mut g, valley, 2, Some(Target::Permanent(cart)));
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.battlefield_find(cart).unwrap().controller, 1);
}
