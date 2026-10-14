//! Commander: two-card gaps of the top-2000 commanders (`decks::cmdr_rank2k_l`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None });
}

/// Arachnus Spinner taps itself to web a creature from the graveyard.
#[test]
fn arachnus_spinner_webs_a_creature() {
    let mut g = pod(2);
    let spinner = ready(&mut g, 0, catalog::arachnus_spinner());
    let web = g.add_card_to_graveyard(0, catalog::arachnus_web());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    act(&mut g, GameAction::ActivateAbility {
        card_id: spinner,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    assert!(g.battlefield_find(spinner).unwrap().tapped, "tapped itself, a Spider");
    assert_eq!(g.battlefield_find(web).expect("on the battlefield").attached_to, Some(bear));
}

/// Gandalf casts a cheap instant for free at the start of combat.
#[test]
fn gandalf_party_guest_casts_free_at_combat() {
    let mut g = pod(2);
    let gandalf = ready(&mut g, 0, catalog::gandalf_party_guest());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let ability = catalog::gandalf_party_guest().triggered_abilities[0].effect.clone();
    g.resolve_effect(&ability, &EffectContext::for_ability(gandalf, 0, None)).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "cast for free");
}

/// Multiversal Incursion copies each nontoken creature, minus legendary.
#[test]
fn multiversal_incursion_copies_the_board() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::gandalf_party_guest());
    ready(&mut g, 0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::multiversal_incursion());
    cast(&mut g, spell, None);
    let copies: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.controller == 0).collect();
    assert_eq!(copies.len(), 2);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Gandalf, Party Guest").count(), 2, "the copy isn't legendary");
}

/// Traverse the Ulvenwald finds a basic land, or a creature with delirium.
#[test]
fn traverse_the_ulvenwald_switches_on_delirium() {
    for delirium in [false, true] {
        let mut g = pod(2);
        g.add_card_to_library(0, catalog::forest());
        let bear = g.add_card_to_library(0, catalog::grizzly_bears());
        if delirium {
            for def in [catalog::island(), catalog::lightning_bolt(), catalog::sol_ring(), catalog::serra_angel()] {
                g.add_card_to_graveyard(0, def);
            }
        }
        if delirium {
            g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
        }
        let traverse = g.add_card_to_hand(0, catalog::traverse_the_ulvenwald());
        cast(&mut g, traverse, None);
        let got_bear = g.players[0].hand.iter().any(|c| c.id == bear);
        assert_eq!(got_bear, delirium);
        assert_eq!(g.players[0].hand.len(), 1);
    }
}
