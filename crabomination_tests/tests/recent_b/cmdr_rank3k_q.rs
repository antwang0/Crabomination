//! Commander: two-card gaps of EDHREC ranks 2501-3500
//! (`decks::cmdr_rank3k_q`).

use crabomination::card::CardId;
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
/// CR 602.5b — Rescue Girl bounces your own permanent, on your turn only.
#[test]
fn rescue_girl_bounces_on_your_turn() {
    let mut g = pod(2);
    let girl = ready(&mut g, 0, catalog::rescue_girl_first_responder());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, girl, 0, Some(Target::Permanent(bear)));
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// CR 615.1 — Morningtide's Light blinks creatures to the end step and shields
/// you until your next turn; then it exiles itself.
#[test]
fn morningtides_light_blinks_and_shields() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let light = g.add_card_to_hand(0, catalog::morningtides_light());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: light,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(angel)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(angel).is_none());
    assert!(g.exile.iter().any(|c| c.id == light), "exiles itself");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "prevented");
    while g.step != TurnStep::End {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(
        g.battlefield
            .iter()
            .filter(|c| c.tapped
                && (c.definition.name == "Grizzly Bears" || c.definition.name == "Serra Angel"))
            .count(),
        2
    );
}
