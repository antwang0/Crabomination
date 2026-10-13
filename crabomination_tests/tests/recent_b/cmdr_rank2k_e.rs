//! Commander: EDHREC ranks 1501-2000, fifth batch (`decks::cmdr_rank2k_e`).

use crabomination::card::CardId;
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

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

/// Namor makes one Merfolk per blue pip of a noncreature spell, and is as
/// strong as your Merfolk.
#[test]
fn namor_counts_blue_pips() {
    let mut g = pod(2);
    let namor = ready(&mut g, 0, catalog::namor_the_sub_mariner());
    assert_eq!(g.computed_permanent(namor).unwrap().power, 1, "Namor is a Merfolk");
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let think = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, think, None);
    assert_eq!(named(&g, "Merfolk"), 1, "{{2}}{{U}}: one pip");
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bears, None);
    assert_eq!(named(&g, "Merfolk"), 1, "a creature spell makes none");
    assert_eq!(g.computed_permanent(namor).unwrap().power, 2);
    assert_eq!(g.computed_permanent(namor).unwrap().toughness, 4);
}

/// Mirror Mockery's attack copy is exiled at end of combat.
#[test]
fn mirror_mockery_copy_leaves_at_end_of_combat() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let mockery = g.add_card_to_hand(0, catalog::mirror_mockery());
    cast(&mut g, mockery, Some(Target::Permanent(bear)));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Grizzly Bears"), 2, "the copy");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(named(&g, "Grizzly Bears"), 1, "the copy is gone");
}

/// Mirage Mockery entwined copies both an artifact creature and a
/// nonartifact one.
#[test]
fn mirage_mockery_entwined_copies_both() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let golem = ready(&mut g, 0, catalog::solemn_simulacrum());
    let mockery = g.add_card_to_hand(0, catalog::mirage_mockery());
    act(
        &mut g,
        GameAction::CastSpellEntwine {
            card_id: mockery,
            target: Some(Target::Permanent(golem)),
            additional_targets: vec![Target::Permanent(bear)],
            mode: None,
            x_value: None,
        },
    );
    assert_eq!((named(&g, "Grizzly Bears"), named(&g, "Solemn Simulacrum")), (2, 2));
}

/// Destined pumps from hand; Lead is cast from the graveyard and exiled.
#[test]
fn destined_lead_is_an_aftermath_split() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let card = g.add_card_to_hand(0, catalog::destined_lead());
    cast(&mut g, card, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == card));
    act(
        &mut g,
        GameAction::CastAftermath {
            card_id: card,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    );
    assert!(g.exile.iter().any(|c| c.id == card), "aftermath exiles it");
}

/// Ellyn Harbreeze looks at as many cards as tokens you made this turn.
#[test]
fn ellyn_harbreeze_digs_by_tokens_created() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let ellyn = ready(&mut g, 0, catalog::ellyn_harbreeze_busybody());
    let goblins = g.add_card_to_hand(0, catalog::you_see_a_pair_of_goblins());
    act(&mut g, GameAction::CastSpell { card_id: goblins, target: None, additional_targets: vec![], mode: Some(1), x_value: None });
    assert_eq!(named(&g, "Goblin"), 2);
    let hand = g.players[0].hand.len();
    act(&mut g, GameAction::ActivateAbility { card_id: ellyn, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None });
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].library.len(), 3, "one of the two looked at went to hand");
}
