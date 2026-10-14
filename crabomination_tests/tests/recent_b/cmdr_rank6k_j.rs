//! Commander: three-card gaps of EDHREC ranks 5501-6500
//! (`decks::cmdr_rank6k_j`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::card::Value;
use crabomination::game::effects::EffectContext;
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

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.players[0].mana_pool.add(Color::Black, 5);
    g.players[0].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

/// CR 121.2 / 701.21a — M.O.D.O.K.: your second draw of the turn makes the
/// opponent sacrifice a nontoken creature; the first draw doesn't.
#[test]
fn modok_second_draw_forces_a_sacrifice() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::m_o_d_o_k_evil_intellect());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::swamp());
    }
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let draw = Effect::Draw { who: Selector::You, amount: Value::ONE };
    for n in 0..2 {
        let evs = g.resolve_effect(&draw, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let alive = g.battlefield.iter().any(|c| c.id == bears);
        assert_eq!(alive, n == 0, "after draw {}", n + 1);
    }
}

/// CR 702.15 / 121.1 — Moment of Defiance: +2/+1, lifelink, and a card.
#[test]
fn moment_of_defiance_pumps_and_draws() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::swamp());
    let s = g.add_card_to_hand(0, catalog::moment_of_defiance());
    cast(&mut g, s, Some(Target::Permanent(bears)));
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3));
    assert!(cp.keywords().contains(&Keyword::Lifelink));
    assert_eq!(g.players[0].hand.len(), 1);
}

/// CR 613.4b / 704.5f — Multiply by Zero sets base 0/0; the creature dies.
#[test]
fn multiply_by_zero_kills_a_vanilla_creature() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::multiply_by_zero());
    cast(&mut g, s, Some(Target::Permanent(bears)));
    assert!(!g.battlefield.iter().any(|c| c.id == bears));
}
