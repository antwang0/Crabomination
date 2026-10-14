//! Commander: four-card gaps of EDHREC ranks 4501-5500 (`decks::cmdr_rank5k_e`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x });
}

fn ability(id: CardId, index: usize, target: Option<Target>) -> GameAction {
    GameAction::ActivateAbility { card_id: id, ability_index: index, target, additional_targets: vec![], x_value: None, mode: None }
}

/// CR 702.193 — Quinjet Technician's {R}{R} funds a power-up activation and
/// nothing else (`SpendRestriction::PowerUpOnly`).
#[test]
fn cr_702_193_quinjet_technician_mana_is_power_up_only() {
    let mut g = pod(2);
    let tech = ready(&mut g, 0, catalog::quinjet_technician());
    let fist = ready(&mut g, 0, catalog::iron_fist_hero_for_hire());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.perform_action(ability(tech, 1, None)).expect("tap for the restricted RR");
    drain_stack(&mut g);
    let sear = g.add_card_to_hand(0, catalog::sear());
    g.players[0].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: sear,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the RR can't pay for Sear's {{R}}"
    );
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(ability(fist, 0, Some(Target::Player(1)))).expect("power-up takes it");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(fist).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
}

/// Structural Assault counts the artifacts it just destroyed.
#[test]
fn structural_assault_scales_with_destroyed_artifacts() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 1, catalog::sol_ring());
    }
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::structural_assault());
    cast(&mut g, id, None, None);
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Sol Ring"));
    assert!(g.battlefield_find(giant).is_none(), "3 damage kills a 3/3");
    assert!(g.battlefield_find(angel).is_some(), "a 4/4 survives 3");
}

/// Dáin Ironfoot arrives with an Axe already on a creature.
#[test]
fn dain_ironfoot_attaches_his_axe() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::dain_ironfoot());
    cast(&mut g, id, None, None);
    let axe = g.battlefield.iter().find(|c| c.definition.name == "Axe").expect("axe");
    assert!(axe.attached_to.is_some());
    let host = axe.attached_to.unwrap();
    assert!(host == bear || host == id);
}

/// Éomer makes a Soldier only while you control the biggest creature.
#[test]
fn eomer_needs_the_greatest_power() {
    let mut g = pod(2);
    let eomer = ready(&mut g, 0, catalog::eomer_of_the_riddermark());
    let attack = |g: &mut GameState| {
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
            attacker: eomer,
            target: crabomination::game::types::AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(g);
    };
    let soldiers = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Human Soldier").count();
    attack(&mut g);
    assert_eq!(soldiers(&g), 1, "5 power is the biggest");
    let mut g2 = pod(2);
    let eomer2 = ready(&mut g2, 0, catalog::eomer_of_the_riddermark());
    ready(&mut g2, 1, catalog::hulk_bruce_banner());
    g2.step = TurnStep::DeclareAttackers;
    g2.priority.player_with_priority = 0;
    g2.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: eomer2,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g2);
    assert_eq!(soldiers(&g2), 0, "the opponent's 7-power Hulk is bigger");
}
