//! Champions of Kamigawa closure — the last six gap cards and the primitives
//! they added.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::two_player_game;
use crabomination::game::types::Target;
use crabomination::game::*;

/// Cast `def` from hand with the mana pre-floated, then drain the stack.
fn cast(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition, target: Option<Target>) -> CardId {
    let id = g.add_card_to_hand(seat, def);
    g.players[seat].mana_pool.add_colorless(20);
    for c in crabomination::mana::Color::ALL {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast spell");
    drain_stack(g);
    id
}

/// CR 611.2c — the shroud grant holds while Hisoka's Guard stays tapped and
/// falls off the moment it untaps.
#[test]
fn hisokas_guard_shroud_lasts_while_it_stays_tapped() {
    let mut g = two_player_game();
    let guard = g.add_card_to_battlefield(0, catalog::hisokas_guard());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(guard).unwrap().summoning_sick = false;
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: guard,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Shroud));

    // Declining the untap keeps the grant alive; untapping ends it.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.do_untap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(guard).unwrap().tapped, "chose not to untap");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Shroud));

    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    g.do_untap();
    g.check_state_based_actions();
    assert!(!g.battlefield_find(guard).unwrap().tapped);
    assert!(
        !g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Shroud),
        "grant ends with the untap"
    );
}

/// Mindblaze pays out only on an exact count, and shuffles either way.
#[test]
fn mindblaze_burns_only_on_an_exact_guess() {
    for (guess, expect_damage) in [(2u32, true), (3u32, false)] {
        let mut g = two_player_game();
        for _ in 0..2 {
            g.add_card_to_library(1, catalog::grizzly_bears());
        }
        g.add_card_to_library(1, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::NamedCard("Grizzly Bears".to_string()),
            DecisionAnswer::Amount(guess),
        ]));
        cast(&mut g, 0, catalog::mindblaze(), Some(Target::Player(1)));
        assert_eq!(
            g.players[1].life == 12,
            expect_damage,
            "guess {guess}: life {}",
            g.players[1].life
        );
    }
}

/// Moonring Mirror stashes drawn-for cards under itself and swaps the stash
/// for the hand on upkeep.
#[test]
fn moonring_mirror_swaps_its_stash_for_your_hand() {
    let mut g = two_player_game();
    let mirror = g.add_card_to_battlefield(0, catalog::moonring_mirror());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let stashed = g
        .exile
        .iter()
        .find(|c| c.exiled_with == Some(mirror))
        .map(|c| c.id)
        .expect("the draw exiled the next card under the Mirror");

    let in_hand: Vec<CardId> = g.players[0].hand.iter().map(|c| c.id).collect();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.id == stashed),
        "the stash came back to hand"
    );
    for id in in_hand {
        assert!(g.exile.iter().any(|c| c.id == id), "the old hand went under the Mirror");
    }
}

/// Reweave sacrifices the target and digs to a card sharing one of its types.
#[test]
fn reweave_replaces_the_sacrificed_permanent_with_a_shared_type() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::forest());
    let hit = g.add_card_to_library(1, catalog::serra_angel());
    cast(&mut g, 0, catalog::reweave(), Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none(), "target was sacrificed");
    assert!(g.battlefield_find(hit).is_some(), "dug to a creature card");
}

/// Struggle for Sanity bins half the hand: the opponent keeps their picks,
/// the caster's go to the graveyard.
#[test]
fn struggle_for_sanity_splits_the_revealed_hand() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let before = g.players[1].hand.len();
    cast(&mut g, 0, catalog::struggle_for_sanity(), Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), before / 2, "opponent kept their picks");
    assert_eq!(g.players[1].graveyard.len(), before / 2, "the caster's picks were binned");
    assert!(g.exile.is_empty(), "nothing stayed in exile");
}

/// CR 612 — Swirl the Mists rewrites every color word to the chosen one, so a
/// protection-from-white creature ends up with protection from the choice.
#[test]
fn swirl_the_mists_rewrites_protection_color_words() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let knight = g.add_card_to_battlefield(1, catalog::stillmoon_cavalier());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    cast(&mut g, 0, catalog::swirl_the_mists(), None);
    let kws = g.computed_permanent(knight).unwrap().keywords;
    assert!(kws.contains(&Keyword::Protection(Color::Blue)), "white and black → blue");
    assert!(!kws.contains(&Keyword::Protection(Color::White)));
    assert!(!kws.contains(&Keyword::Protection(Color::Black)));
}
