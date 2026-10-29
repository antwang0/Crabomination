//! Commander: two-card gaps of EDHREC ranks 2501-3500
//! (`decks::cmdr_rank3k_k`).

use crabomination::card::{CardId, Keyword};
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

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
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
/// CR 207.2c — Raphael's enrage adds red mana that survives step changes.
#[test]
fn raphael_enrage_adds_kept_red() {
    let mut g = pod(2);
    let raph = ready(&mut g, 0, catalog::raphael_ninja_destroyer());
    g.players[0].mana_pool.empty();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(raph)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3);
    let evs = g.advance_step(Vec::new()).unwrap();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.players[0].mana_pool.total(), 3, "kept as the step ends");
    assert!(
        g.computed_permanent(raph)
            .unwrap()
            .keywords()
            .contains(&Keyword::MustBeBlocked)
    );
}

/// CR 702.190 — Raphael's Technique offers each player a fresh seven.
#[test]
fn raphaels_technique_wheels() {
    let mut g = pod(2);
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::raphaels_technique());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, t, None);
    assert_eq!(g.players[0].hand.len(), 7);
    assert!(
        g.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition.name == "Grizzly Bears")
    );
}

/// CR 603.3d — Rinoa returns the first attacking creature to die each turn,
/// tapped, with a flying counter; her combat trigger pumps flyers.
#[test]
fn rinoa_returns_a_fallen_attacker() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::rinoa_angel_wing());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let angel = ready(&mut g, 0, catalog::serra_angel());
    g.step = TurnStep::PreCombatMain;
    let evs = g.advance_step(Vec::new()).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.step, TurnStep::BeginCombat);
    assert_eq!(pt(&g, angel), (5, 5));
    declare(&mut g, vec![(bear, 1)]);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    kill(&mut g, bear);
    let back = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Grizzly Bears")
        .expect("returned");
    assert!(back.tapped);
    assert!(
        g.computed_permanent(back.id)
            .unwrap()
            .keywords()
            .contains(&Keyword::Flying)
    );
}

/// "Do this only once each turn" is spent by the return, not the trigger: a
/// declined death leaves the turn's return for the next, which then spends it.
#[test]
fn rinoa_declined_return_keeps_the_turns_use() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::rinoa_angel_wing());
    let bears: Vec<_> = (0..3).map(|_| ready(&mut g, 0, catalog::grizzly_bears())).collect();
    g.step = TurnStep::BeginCombat;
    declare(&mut g, bears.iter().map(|&b| (b, 1)).collect());
    let returned = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    for (b, take, left) in [(bears[0], false, 2), (bears[1], true, 2), (bears[2], true, 1)] {
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(take)]));
        kill(&mut g, b);
        assert_eq!(returned(&g), left, "after {b:?} (take {take})");
    }
}

/// CR 601.3b — Fated Clash is a sorcery that can be cast mid-combat once a
/// creature attacks and another blocks; it spares its two targets.
#[test]
fn fated_clash_wipes_but_spares_two() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let other = ready(&mut g, 0, catalog::serra_angel());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let doomed = ready(&mut g, 1, catalog::serra_angel());
    let clash = g.add_card_to_hand(0, catalog::fated_clash());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: clash,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("main-phase cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_some() && g.battlefield_find(theirs).is_some());
    assert!(g.battlefield_find(other).is_none() && g.battlefield_find(doomed).is_none());
    let late = g.add_card_to_hand(0, catalog::fated_clash());
    g.step = TurnStep::DeclareBlockers;
    flood(&mut g);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: late,
            target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "no attacker or blocker: sorcery timing only"
    );
}
