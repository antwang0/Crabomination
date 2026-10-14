//! Commander: one-card gaps of EDHREC ranks 3501-4500 (`decks::cmdr_rank4k_j`).

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
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: x,
        },
    );
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .count()
}


/// Karn, Argent Defender: an entering creature or artifact triggers nothing,
/// its own ETB included.
#[test]
fn karn_argent_defender_silences_arrivals() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::karn_argent_defender());
    let kiora = g.add_card_to_hand(0, catalog::kiora_of_fire_and_ashes());
    cast(&mut g, kiora, None, None);
    assert_eq!(named(&g, 0, "Dragon"), 0);
}

/// Ward of Bones: an opponent ahead of its controller on creatures can't cast
/// a creature spell, and one ahead on lands can't play a land; the controller
/// itself is never bound.
#[test]
fn ward_of_bones_binds_the_opponent_ahead() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::ward_of_bones());
    ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::forest());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 2);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "one creature to none: no creature spell"
    );
    assert!(!g.can_player_play_land(1), "one land to none: no land drop");
    assert!(g.can_player_play_land(0));
    let ring = g.add_card_to_hand(1, catalog::sol_ring());
    g.players[1].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: ring,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_ok(),
        "behind on artifacts (the Ward is one), an artifact spell is free to cast"
    );
}

/// CR 614.1a / 111.7 — Hallowed Moonlight: a creature entering without being
/// cast is exiled, and a creature token is never made; a cast creature
/// enters as usual.
#[test]
fn hallowed_moonlight_exiles_the_uncast() {
    let mut g = pod(2);
    let hm = g.add_card_to_hand(0, catalog::hallowed_moonlight());
    g.add_card_to_library(0, catalog::island());
    cast(&mut g, hm, None, None);
    let kiora = g.add_card_to_hand(0, catalog::kiora_of_fire_and_ashes());
    cast(&mut g, kiora, None, None);
    assert!(g.battlefield_find(kiora).is_some(), "cast, so it enters");
    assert_eq!(named(&g, 0, "Dragon"), 0, "the token is never made");
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &crabomination::effect::Effect::Move {
            what: crabomination::effect::Selector::ExactObjects(vec![bear]),
            to: crabomination::effect::ZoneDest::Battlefield {
                controller: crabomination::effect::PlayerRef::You,
                tapped: false,
            },
        },
        &ctx,
    )
    .unwrap();
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.exile.iter().any(|c| c.id == bear), "reanimated, so exiled");
}

/// Sigrid flashes in and holds an attacker until she leaves.
#[test]
fn sigrid_holds_an_attacker() {
    let mut g = pod(2);
    let att = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: att,
        target: crabomination::game::types::AttackTarget::Player(0),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let s = g.add_card_to_hand(0, catalog::sigrid_god_favored());
    cast(&mut g, s, None, None);
    assert!(g.battlefield_find(att).is_none());
    assert!(g.exile.iter().any(|c| c.id == att));
}

/// CR 104.2a / 104.3d — the last seat standing wins at negative life while
/// its Platinum Angel keeps it in the game (pod sweep 441342, game 1).
#[test]
fn platinum_angel_seat_wins_as_last_standing() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::platinum_angel());
    g.players[0].life = -6;
    g.players[2].life = -3;
    g.check_state_based_actions();
    assert!(g.game_over.is_none());
    g.players[1].life = -17;
    g.check_state_based_actions();
    assert_eq!(g.game_over, Some(Some(0)));
}
