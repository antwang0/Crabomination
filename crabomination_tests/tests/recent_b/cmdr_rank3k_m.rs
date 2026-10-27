//! Commander: two-card gaps of EDHREC ranks 2501-3500
//! (`decks::cmdr_rank3k_m`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
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
/// CR 903.3d — Sifa Grent can be a commander; deaths drain, +1 goads two.
#[test]
fn sifa_grent_drains_and_goads() {
    let mut g = pod(3);
    assert!(catalog::sifa_grent().can_be_commander);
    let sifa = ready(&mut g, 0, catalog::sifa_grent());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let other = ready(&mut g, 2, catalog::grizzly_bears());
    let plus = catalog::sifa_grent().loyalty_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    ctx.targets.push(Target::Permanent(other));
    ctx.source = Some(sifa);
    let evs = g.resolve_effect(&plus, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    assert!(
        g.goaded_by_player(g.battlefield_find(bear).unwrap(), 0)
            && g.goaded_by_player(g.battlefield_find(other).unwrap(), 0)
    );
    kill(&mut g, bear);
    assert_eq!(g.players[0].life, 21);
    assert_eq!(g.players[1].life + g.players[2].life, 39);
}

/// CR 701.38 — Seifer goads a creature of the player he attacks.
#[test]
fn seifer_goads_the_attacked_player() {
    let mut g = pod(3);
    let seifer = ready(&mut g, 0, catalog::seifer_balamb_rival());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    declare(&mut g, vec![(seifer, 1)]);
    assert!(g.goaded_by_player(g.battlefield_find(bear).unwrap(), 0));
}

/// Seifer: ANY creature attacking one of your opponents gains deathtouch when
/// two or more creatures block it — another player's attacker included; one
/// attacking you doesn't.
#[test]
fn seifer_grants_deathtouch_to_any_double_blocked_attacker_of_an_opponent() {
    for (defender, expect) in [(2usize, true), (0, false)] {
        let mut g = pod(3);
        ready(&mut g, 0, catalog::seifer_balamb_rival());
        let raider = ready(&mut g, 1, catalog::craw_wurm());
        let a = ready(&mut g, defender, catalog::grizzly_bears());
        let b = ready(&mut g, defender, catalog::grizzly_bears());
        g.active_player_idx = 1;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: raider, target: AttackTarget::Player(defender) }]))
            .expect("attack");
        drain_stack(&mut g);
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = defender;
        g.perform_action(GameAction::DeclareBlockers(vec![(a, raider), (b, raider)])).expect("double block");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(raider).unwrap().keywords().contains(&Keyword::Deathtouch), expect, "defender {defender}");
    }
}

/// CR 506.4 — Nivea pulls a blocked creature out of combat, untapped.
#[test]
fn nivea_rescues_a_blocked_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::nivea_beloved_battlemage());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let wall = ready(&mut g, 1, catalog::serra_angel());
    declare(&mut g, vec![(bear, 1)]);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, bear)]))
        .expect("block");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert!(!c.tapped);
    finish_combat(&mut g);
    assert!(
        g.battlefield_find(bear).is_some(),
        "out of combat, never dealt damage"
    );
}

/// CR 111.1 — Nivea's sacrifice makes an Akroma token.
#[test]
fn nivea_becomes_akroma() {
    let mut g = pod(2);
    let nivea = ready(&mut g, 0, catalog::nivea_beloved_battlemage());
    activate(&mut g, nivea, 0, None);
    let akroma = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Akroma, Angel of Wrath")
        .expect("token")
        .id;
    assert_eq!(pt(&g, akroma), (6, 6));
    // CR 111.3 — the token's defined {5}{W}{W}{W} is its mana cost.
    assert_eq!(g.battlefield_find(akroma).unwrap().definition.cost.cmc(), 8);
    assert!(
        g.computed_permanent(akroma)
            .unwrap()
            .keywords()
            .contains(&Keyword::Protection(Color::Red))
    );
}

/// CR 702.149a — Warrior's Resolve grants training and draws for a veteran.
#[test]
fn warriors_resolve_trains_and_draws() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::warriors_resolve());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let angel = ready(&mut g, 0, catalog::serra_angel());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    declare(&mut g, vec![(bear, 1), (angel, 1)]);
    assert_eq!(plus(&g, bear), 1, "trained alongside the Angel");
    finish_combat(&mut g);
    let hand = g.players[0].hand.len();
    while g.step != TurnStep::End {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1);
}
