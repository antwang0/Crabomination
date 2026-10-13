//! Commander: Princess Yue and Etrata's gaps (`decks::cmdr_edhrec54`).

use crabomination::card::{CardId, Keyword};
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

fn fill_pool(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
    g.priority.player_with_priority = 0;
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn kill(g: &mut GameState, id: CardId) {
    for seat in 0..2 {
        g.add_card_to_library(seat, catalog::island());
    }
    let mut evs = Vec::new();
    g.destroy_permanent(id, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// Princess Yue dies and returns tapped as a legendary land named Moon with
/// "{T}: Add {C}"; the land can't die, so it doesn't come back again.
#[test]
fn princess_yue_returns_as_the_moon() {
    let mut g = pod(2);
    let yue = ready(&mut g, 0, catalog::princess_yue());
    kill(&mut g, yue);
    let moon = g.battlefield.iter().find(|c| c.id == yue).expect("back as a land");
    assert!(moon.tapped);
    let cp = g.computed_permanent(yue).unwrap();
    assert_eq!(cp.def.name, "Moon");
    assert!(cp.def.is_land() && !cp.def.is_creature());
    assert_eq!(cp.def.activated_abilities.len(), 2, "scry 2 and {{T}}: Add {{C}}");
}

/// Hulking Metamorph cast for its prototype copies a creature you control but
/// stays 3/3 and becomes an artifact (CR 702.160 + 707.9b).
#[test]
fn hulking_metamorph_copies_at_its_own_size() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::colossal_dreadmaw());
    let meta = g.add_card_to_hand(0, catalog::hulking_metamorph());
    fill_pool(&mut g);
    g.perform_action(GameAction::CastPrototype {
        card_id: meta,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("prototype cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(meta).expect("on the battlefield");
    assert_eq!(cp.def.name, "Colossal Dreadmaw");
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.def.is_artifact());
    assert!(cp.keywords().contains(&Keyword::Trample), "the copied ability");
}

/// Guildsworn Prowler draws when it dies outside a block, not as a blocker
/// (CR 603.10 — the blocker that just died still reads as blocking).
#[test]
fn guildsworn_prowler_draws_unless_it_was_blocking() {
    let mut g = pod(2);
    let prowler = ready(&mut g, 0, catalog::guildsworn_prowler());
    let hand = g.players[0].hand.len();
    kill(&mut g, prowler);
    assert_eq!(g.players[0].hand.len(), hand + 1);

    let mut g = pod(2);
    let prowler = ready(&mut g, 1, catalog::guildsworn_prowler());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: giant, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(prowler, giant)])).expect("block");
    drain_stack(&mut g);
    let hand = g.players[1].hand.len();
    kill(&mut g, prowler);
    assert_eq!(g.players[1].hand.len(), hand, "it was blocking");
}

/// Hookblade Veteran flies only during its controller's turn.
#[test]
fn hookblade_veteran_flies_on_your_turn() {
    let mut g = pod(2);
    let vet = ready(&mut g, 0, catalog::hookblade_veteran());
    assert!(g.computed_permanent(vet).unwrap().keywords().contains(&Keyword::Flying));
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(vet).unwrap().keywords().contains(&Keyword::Flying));
}

/// Become Anonymous exiles your creature and the top two cards, then cloaks
/// all three face down and tapped as 2/2s with ward {2} (CR 702.182).
#[test]
fn become_anonymous_cloaks_the_pile() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].library.clear();
    let top: Vec<CardId> = (0..2).map(|_| g.add_card_to_library(0, catalog::forest())).collect();
    let spell = g.add_card_to_hand(0, catalog::become_anonymous());
    fill_pool(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    for id in std::iter::once(bears).chain(top) {
        let c = g.battlefield.iter().find(|c| c.id == id).expect("cloaked");
        assert!(c.face_down && c.tapped, "face down and tapped");
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2));
    }
}
