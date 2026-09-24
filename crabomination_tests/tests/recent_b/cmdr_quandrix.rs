//! Commander: the Quantum Quandrix precon (C21, Adrix and Nev,
//! `decks::cmdr_quandrix`) and the primitives it needed.

use crabomination::card::{
    CardDefinition, CardId, CardType, CounterType, CreatureType, SelectionRequirement as R,
    StaticAbility, Subtypes, Value,
};
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, StaticEffect};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;
use std::sync::Arc;

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

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn soldiers(n: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: Arc::new(crabomination::card::TokenDefinition {
            name: "Soldier".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
            ..Default::default()
        }),
    }
}

fn run(g: &mut GameState, seat: usize, e: &Effect) {
    let ctx = EffectContext::for_spell(seat, None, 0, 0);
    g.resolve_effect(e, &ctx).expect("resolve");
    drain_stack(g);
}

/// CR 614 — the first token batch on the static's controller's turn becomes
/// that many copies of the greatest-mana-value creature other than its
/// source; the second batch, and a batch on another player's turn, don't.
#[test]
fn cr_614_first_tokens_on_your_turn_become_copies_of_the_chosen_creature() {
    let mut g = pod(3);
    let esix_like = CardDefinition {
        name: "Test Bloom",
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "first tokens are copies",
            effect: StaticEffect::FirstTokensOnYourTurnBecomeCopiesOfChosen,
        }],
        ..Default::default()
    };
    g.add_card_to_battlefield(0, esix_like);
    g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    run(&mut g, 0, &soldiers(2));
    assert_eq!(named(&g, 0, "Serra Angel").len(), 2, "two copies of the biggest creature, anyone's");
    assert!(named(&g, 0, "Soldier").is_empty());
    run(&mut g, 0, &soldiers(1));
    assert_eq!(named(&g, 0, "Soldier").len(), 1, "once a turn");

    let mut g = pod(3);
    g.active_player_idx = 1;
    g.add_card_to_battlefield(
        0,
        CardDefinition {
            name: "Test Bloom",
            card_types: vec![CardType::Creature],
            static_abilities: vec![StaticAbility {
                description: "first tokens are copies",
                effect: StaticEffect::FirstTokensOnYourTurnBecomeCopiesOfChosen,
            }],
            ..Default::default()
        },
    );
    g.add_card_to_battlefield(1, catalog::serra_angel());
    run(&mut g, 0, &soldiers(1));
    assert_eq!(named(&g, 0, "Soldier").len(), 1, "only during your turn");
}

/// CR 614 — after the steal, a token an opponent would create this turn is
/// created under the thief's control (and owned by it, CR 111.2).
#[test]
fn cr_614_an_opponents_tokens_are_created_under_the_thiefs_control() {
    let mut g = pod(3);
    run(&mut g, 0, &Effect::StealOpponentTokensThisTurn);
    run(&mut g, 2, &soldiers(2));
    assert_eq!(named(&g, 0, "Soldier").len(), 2);
    assert!(named(&g, 2, "Soldier").is_empty());
    run(&mut g, 0, &soldiers(1));
    assert_eq!(named(&g, 0, "Soldier").len(), 3, "your own tokens stay yours");
}

/// CR 603.4 — "whenever a nontoken creature an opponent controls enters this
/// turn": an opponent's cast creature fires it, the entering creature is the
/// trigger source; the watcher's own creature doesn't.
#[test]
fn cr_603_4_a_turn_scoped_trigger_watches_other_players_creatures_enter() {
    let mut g = pod(3);
    run(
        &mut g,
        0,
        &Effect::WheneverCreatureEntersThisTurn {
            filter: R::Creature.and(R::ControlledByOpponent).and(R::NotToken),
            body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
        },
    );
    let life = g.players[0].life;
    let bear = g.add_card_to_hand(2, catalog::grizzly_bears());
    g.active_player_idx = 2;
    g.priority.player_with_priority = 2;
    flood(&mut g, 2);
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1);
    let mine = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: mine, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "not your own creature");
}
