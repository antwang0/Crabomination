//! Commander: single cards completing several top-2000 decks
//! (`decks::cmdr_rank2k_i`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
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

fn chapter(def: &crabomination::card::CardDefinition, n: u32) -> crabomination::effect::Effect {
    def.saga_chapters.iter().find(|(c, _)| *c == n).unwrap().1.clone()
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

/// Attuma pumps other Merfolk and draws once per player its Merfolk attack.
#[test]
fn attuma_draws_per_attacked_player() {
    let mut g = pod(3);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    ready(&mut g, 0, catalog::attuma_atlantean_warlord());
    let a = ready(&mut g, 0, catalog::silvergill_adept());
    let b = ready(&mut g, 0, catalog::silvergill_adept());
    assert_eq!(g.computed_permanent(a).unwrap().power, 3);
    let hand = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Jin-Gitaxias draws off a big noncreature spell, then flips with a full
/// hand; chapter I doubles the hand.
#[test]
fn jin_gitaxias_draws_and_flips() {
    let mut g = pod(2);
    for _ in 0..20 {
        g.add_card_to_library(0, catalog::island());
    }
    let jin = ready(&mut g, 0, catalog::jin_gitaxias());
    let div = g.add_card_to_hand(0, catalog::divination());
    act(&mut g, GameAction::CastSpell { card_id: div, target: None, additional_targets: vec![], mode: None, x_value: None });
    assert_eq!(g.players[0].hand.len(), 3, "Divination's two plus Jin's one");
    let activate = GameAction::ActivateAbility {
        card_id: jin,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    };
    flood(&mut g);
    assert!(g.perform_action(activate.clone()).is_err(), "needs seven cards in hand");
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::island());
    }
    act(&mut g, activate);
    assert_eq!(g.battlefield_find(jin).map(|c| c.definition.name), Some("The Great Synthesis"));
    assert_eq!(g.players[0].hand.len(), 14);
}

/// The Great Synthesis II bounces non-Phyrexians; III casts from hand for
/// free and returns Jin-Gitaxias.
#[test]
fn the_great_synthesis_bounces_then_casts_free() {
    let mut g = pod(2);
    let synthesis = *catalog::jin_gitaxias().back_face.unwrap();
    let host = ready(&mut g, 0, catalog::jin_gitaxias());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let rager = ready(&mut g, 1, catalog::phyrexian_rager());
    let ctx = EffectContext::for_ability(host, 0, None);
    g.resolve_effect(&chapter(&synthesis, 2), &ctx).unwrap();
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(rager).is_some());
    assert!(g.battlefield_find(host).is_some(), "Jin-Gitaxias is a Phyrexian");
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    g.resolve_effect(&chapter(&synthesis, 3), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_some(), "cast for free");
}

/// The Hunger Tide Rises makes Insects, then trades two of them for a
/// two-drop from the library.
#[test]
fn the_hunger_tide_rises_trades_insects_for_a_creature() {
    let mut g = pod(2);
    let def = catalog::the_hunger_tide_rises();
    let host = ready(&mut g, 0, catalog::the_hunger_tide_rises());
    let ctx = EffectContext::for_ability(host, 0, None);
    for n in 1..=2 {
        g.resolve_effect(&chapter(&def, n), &ctx).unwrap();
    }
    assert_eq!(named(&g, "Insect"), 2);
    g.add_card_to_library(0, catalog::serra_angel());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    g.resolve_effect(&chapter(&def, 4), &ctx).unwrap();
    assert_eq!(named(&g, "Insect"), 0);
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(named(&g, "Serra Angel"), 0, "mana value 5 is over the two sacrificed");
}
