//! Commander: the 2026-10-18 (`019Y6J4R`) fifteenth closer batch
//! (`decks::cmdr_019y_o`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}


use crabomination::game::types::{GameAction, Target, TurnStep};

/// Teferi's Talent: each card you draw adds a loyalty counter to the enchanted
/// planeswalker (CR 306.5c counters), and the walker gains the −12.
#[test]
fn teferis_talent_charges_on_draws() {
    let mut g = pod(2);
    let lili = ready(&mut g, 0, catalog::liliana_death_mage());
    let start = g.battlefield_find(lili).unwrap().counter_count(CounterType::Loyalty);
    let tt = g.add_card_to_hand(0, catalog::teferis_talent());
    flood(&mut g, 0);
    cast(&mut g, tt, Some(Target::Permanent(lili))).expect("cast");
    let draw = crabomination::effect::Effect::Draw {
        who: crabomination::effect::Selector::You,
        amount: crabomination::card::Value::Const(2),
    };
    let evs = g.resolve_effect(&draw, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(lili).unwrap().counter_count(CounterType::Loyalty), start + 2);
}

#[allow(dead_code)]
fn _unused(_: TurnStep, _: Option<GameAction>) {}

/// The Clone Saga II copies the next creature spell as a nonlegendary token
/// (CR 707.9b); III names a creature and draws when it connects (CR 603.7).
#[test]
fn the_clone_saga_copies_then_draws_on_the_named_creature() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::the_clone_saga());
    g.saga_advance(saga);
    drain_stack(&mut g);
    g.saga_advance(saga);
    drain_stack(&mut g);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    cast(&mut g, bear, None).expect("bear");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears").count(),
        2,
        "the bear and its copy"
    );
    g.saga_advance(saga);
    drain_stack(&mut g);
    for c in g.battlefield.iter_mut() {
        c.summoning_sick = false;
    }
    let hand = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: bear,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::End && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
        drain_stack(&mut g);
        if g.step == TurnStep::PostCombatMain {
            break;
        }
    }
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the named Bear connected");
}
