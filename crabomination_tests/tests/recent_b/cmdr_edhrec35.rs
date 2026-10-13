//! Commander: Whtz's deck cards (`decks::cmdr_edhrec35`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

/// Lost Auramancers fetches an enchantment only when it dies with no time
/// counters left (vanishing ran out).
#[test]
fn lost_auramancers_needs_empty_vanishing() {
    let mut g = pod(2);
    let aura = g.add_card_to_library(0, catalog::protective_bubble());
    ready(&mut g, 0, catalog::grizzly_bears());
    let lost = ready(&mut g, 0, catalog::lost_auramancers());
    g.battlefield_find_mut(lost).unwrap().counters = Default::default();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::Destroy {
                what: crabomination::effect::Selector::ExactObjects(vec![lost]),
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(aura).is_some(),
        "lib {} gy {:?} lost {:?}",
        g.players[0].library.len(),
        g.players[0].graveyard.iter().map(|c| c.definition.name).collect::<Vec<_>>(),
        g.battlefield_find(lost).is_some()
    );
}

/// Reed Richards: the turn's first draw outside your draw step is four.
#[test]
fn reed_richards_first_extra_draw_is_four() {
    let mut g = pod(2);
    library(&mut g, 0, 10);
    ready(&mut g, 0, catalog::reed_richards_smartest_man());
    // This turn's draw-step draw has happened.
    g.players[0].cards_drawn_this_turn = 1;
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt, &[]).expect("opt");
    assert_eq!(g.players[0].hand.len(), 4);
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt, &[]).expect("opt");
    assert_eq!(g.players[0].hand.len(), 5, "only the first one each turn");
}

