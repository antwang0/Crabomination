//! CR conformance for this run's engine work:
//! - CR 100 — deck construction: sideboard size and the combined four-of.
//! - CR 119 — life gain punished into a loss, without re-punishing the loss.
//! - CR 509.1b — a block restriction keyed to the defender's biggest tribe.
//! - CR 701.19a — a search whose picker and whose library differ.

use crabomination::card::CardDefinition;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::format::{Deck, DeckError, Format, validate_full_deck};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn legal_modern_main() -> Vec<CardDefinition> {
    let mut main: Vec<CardDefinition> = (0..4).map(|_| catalog::grizzly_bears()).collect();
    main.extend((0..56).map(|_| catalog::forest()));
    main
}

/// CR 100.4a — the sideboard is capped at fifteen cards.
#[test]
fn cr_100_4a_sideboard_is_capped() {
    let deck = Deck {
        main: legal_modern_main(),
        commanders: Vec::new(),
        sideboard: (0..4)
            .map(|_| catalog::llanowar_elves())
            .chain((0..12).map(|_| catalog::forest()))
            .collect(),
    };
    let errs = validate_full_deck(&deck, Format::Modern).expect_err("over the cap");
    assert!(errs.iter().any(|e| matches!(
        e,
        DeckError::SideboardTooLarge { found: 16, maximum: 15 }
    )));

    let deck = Deck { sideboard: deck.sideboard[..15].to_vec(), ..deck };
    assert!(validate_full_deck(&deck, Format::Modern).is_ok());
}

/// CR 100.2a — the four-of limit counts main deck and sideboard together.
#[test]
fn cr_100_2a_copies_count_the_sideboard() {
    let deck = Deck {
        main: legal_modern_main(),
        commanders: Vec::new(),
        sideboard: vec![catalog::grizzly_bears()],
    };
    let errs = validate_full_deck(&deck, Format::Modern).expect_err("five Bears in all");
    assert!(errs.iter().any(|e| matches!(
        e,
        DeckError::TooManyCopies { card_name: "Grizzly Bears", found: 5, maximum: 4 }
    )));

    // Basic lands are exempt from the limit in either zone (CR 100.2a).
    let deck = Deck {
        sideboard: (0..15).map(|_| catalog::forest()).collect(),
        ..deck
    };
    assert!(validate_full_deck(&deck, Format::Modern).is_ok());
}

/// CR 119.3/119.7 — False Cure turns a gain into a bigger loss, and the loss
/// it causes isn't itself punished.
#[test]
fn cr_119_life_gain_punish_does_not_recurse() {
    let mut g = main_phase();
    let cure = g.add_card_to_hand(0, catalog::false_cure());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: cure,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let before = g.players[0].life;
    g.adjust_life(0, 4);
    assert_eq!(g.players[0].life, before + 4 - 8, "net -4, not an infinite spiral");
}

/// CR 509.1b — Graxiplon can't be blocked unless the defender controls three
/// creatures that share a creature type.
#[test]
fn cr_509_1b_block_gated_on_the_defenders_tribe() {
    let mut g = main_phase();
    let grax = g.add_card_to_battlefield(0, catalog::graxiplon());
    g.clear_sickness(grax);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // A third creature of a *different* type doesn't unlock the block.
    g.add_card_to_battlefield(1, catalog::llanowar_elves());
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: grax,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(a, grax)])).is_err());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(a, grax)])).is_ok());
}

/// CR 701.19a — Head Games searches the *opponent's* library, but the caster
/// makes the picks.
#[test]
fn cr_701_19a_searcher_and_library_can_differ() {
    let mut g = main_phase();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let picked = g.add_card_to_library(1, catalog::llanowar_elves());
    g.add_card_to_library(1, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Cards(vec![picked])]));
    let games = g.add_card_to_hand(0, catalog::head_games());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: games,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 1);
    assert_eq!(g.players[1].hand[0].id, picked, "the caster chose it");
}

/// CR 702.73a — a changeling counts toward every tribe for Graxiplon's gate.
#[test]
fn cr_702_73a_changelings_fill_any_tribe() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::llanowar_elves());
    assert_eq!(g.greatest_shared_type_count(1), 1);
    g.add_card_to_battlefield(1, catalog::changeling_hero());
    assert_eq!(
        g.greatest_shared_type_count(1),
        2,
        "the changeling joins whichever tribe is biggest"
    );
}
