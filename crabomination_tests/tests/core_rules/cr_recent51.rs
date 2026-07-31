//! CR conformance for the SOK wave:
//! - CR 307.1/307.4 — sorcery cast timing and "sorceries can't enter the
//!   battlefield".
//! - CR 603.8 — state-triggered flip (Rune-Tail) fires as soon as the game
//!   state matches and doesn't re-fire once flipped.
//! - CR 207.2c — Sweep and Channel are ability words: they carry no rules
//!   meaning of their own, so a Sweep with nothing to return still resolves.

use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
}

/// CR 307.1 — a sorcery is castable only during a main phase of its
/// controller's turn.
#[test]
fn cr_307_1_sorcery_needs_your_main_phase() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::presence_of_the_wise());
    g.players[0].mana_pool.add(Color::White, 4);
    g.step = TurnStep::Upkeep;
    assert!(cast(&mut g, spell, None).is_err(), "not a main phase");
    g.step = TurnStep::PreCombatMain;
    assert!(cast(&mut g, spell, None).is_ok());
}

/// CR 307.1 — a sorcery is castable only while the stack is empty.
#[test]
fn cr_307_1_sorcery_needs_an_empty_stack() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt on the stack");
    let spell = g.add_card_to_hand(0, catalog::presence_of_the_wise());
    g.players[0].mana_pool.add(Color::White, 4);
    assert!(cast(&mut g, spell, None).is_err(), "the stack isn't empty");
    drain_stack(&mut g);
    assert!(cast(&mut g, spell, None).is_ok());
}

/// CR 307.4 — a sorcery that would enter the battlefield stays where it is.
#[test]
fn cr_307_4_sorcery_cant_enter_the_battlefield() {
    let mut g = two_player_game();
    let spell = g.add_card_to_graveyard(0, catalog::presence_of_the_wise());
    let mut events = vec![];
    g.move_card_to(
        spell,
        &crabomination::effect::ZoneDest::Battlefield {
            controller: crabomination::effect::PlayerRef::You,
            tapped: false,
        },
        &crabomination::game::effects::EffectContext::for_ability(spell, 0, None),
        &mut events,
    );
    assert!(g.battlefield_find(spell).is_none(), "still in the graveyard");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == spell));
}

/// CR 603.8 — the state trigger fires the moment the condition holds, and
/// flipping clears it so it never fires twice.
#[test]
fn cr_603_8_state_triggered_flip_fires_once() {
    let mut g = two_player_game();
    let fox = g.add_card_to_battlefield(0, catalog::rune_tail_kitsune_ascendant());
    g.players[0].life = 29;
    g.check_state_based_actions();
    assert!(!g.battlefield_find(fox).unwrap().flipped);
    g.players[0].life = 30;
    g.check_state_based_actions();
    assert!(g.battlefield_find(fox).unwrap().flipped);
    let name = g.battlefield_find(fox).unwrap().definition.name;
    g.players[0].life = 40;
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(fox).unwrap().definition.name, name, "no second flip");
}

/// CR 207.2c — Sweep is an ability word: with no Mountains to return, the
/// spell still resolves and simply deals zero.
#[test]
fn cr_207_2c_sweep_ability_word_has_no_rules_meaning() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::barrel_down_sokenzan());
    g.players[0].mana_pool.add(Color::Red, 3);
    cast(&mut g, spell, Some(Target::Permanent(bear))).expect("still castable");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == spell), "it resolved");
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0);
}
