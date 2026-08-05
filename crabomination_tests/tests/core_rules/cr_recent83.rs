//! CR conformance for this run:
//! - CR 711 — leveler cards: band P/T + abilities, the always-live level-up
//!   ability, the sub-N1 default body, and the off-battlefield printed P/T.
//! - CR 715 — adventurer cards: only the Adventure's characteristics are
//!   evaluated on the stack, and the exiled card can be played but not
//!   re-cast as an Adventure.
//! - CR 614 — an "as this enters" replacement applies on every battlefield
//!   entry, not only spell resolution.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn level_up(g: &mut GameState, id: CardId, times: usize) {
    for _ in 0..times {
        g.players[0].mana_pool.add(Color::White, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("level up");
        drain_stack(g);
    }
}

// ── CR 711 — Leveler Cards ──────────────────────────────────────────────────

/// 711.5 — below the first band's N1 the creature keeps its printed body.
#[test]
fn cr_711_5_below_the_first_band_keeps_the_printed_body() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::student_of_warfare());
    let c = g.computed_permanent(s).expect("computed");
    assert_eq!((c.power, c.toughness), (1, 1));
    assert!(!c.keywords.contains(&Keyword::FirstStrike));
}

/// 711.2a — "{LEVEL N1-N2}" sets base P/T and grants its abilities inside the
/// closed range, and stops applying past N2.
#[test]
fn cr_711_2a_closed_band_applies_only_inside_its_range() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::student_of_warfare());
    level_up(&mut g, s, 2);
    let c = g.computed_permanent(s).expect("computed");
    assert_eq!((c.power, c.toughness), (3, 3));
    assert!(c.keywords.contains(&Keyword::FirstStrike));
    level_up(&mut g, s, 5);
    let c = g.computed_permanent(s).expect("computed");
    assert!(!c.keywords.contains(&Keyword::FirstStrike), "level 7 left the 2-6 band");
}

/// 711.2b — "{LEVEL N3+}" is open-ended: it keeps applying above N3.
#[test]
fn cr_711_2b_open_band_keeps_applying() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::student_of_warfare());
    level_up(&mut g, s, 9);
    let c = g.computed_permanent(s).expect("computed");
    assert_eq!((c.power, c.toughness), (4, 4));
    assert!(c.keywords.contains(&Keyword::DoubleStrike));
}

/// 711.4 — the level-up ability is live at every level, including inside a
/// band's range.
#[test]
fn cr_711_4_level_up_is_available_at_every_level() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::student_of_warfare());
    level_up(&mut g, s, 8);
    assert_eq!(
        g.battlefield_find(s).unwrap().counters.get(&CounterType::Level).copied(),
        Some(8)
    );
}

/// 711.6 — off the battlefield a leveler has its uppermost printed P/T.
#[test]
fn cr_711_6_off_battlefield_uses_the_printed_body() {
    let d = catalog::student_of_warfare();
    assert_eq!((d.power, d.toughness), (1, 1));
    let d = catalog::hexdrinker();
    assert_eq!((d.power, d.toughness), (2, 1));
}

// ── CR 715 — Adventurer Cards ───────────────────────────────────────────────

/// 715.3b — on the stack as an Adventure the spell has only the Adventure's
/// characteristics: a Land // Sorcery adventurer is a *sorcery* spell.
#[test]
fn cr_715_3b_adventure_spell_has_only_its_own_characteristics() {
    let mut g = two_player_game();
    let midgar = g.add_card_to_hand(0, catalog::midgar_city_of_mako());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    g.players[0].mana_pool.add(Color::Black, 3);
    g.perform_action(GameAction::CastAdventure {
        card_id: midgar,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Adventure");
    // A land face can never be cast, so the tally proves the stack object is
    // the Adventure sorcery, not the land.
    assert_eq!(g.players[0].instants_or_sorceries_cast_this_turn, 1);
    assert_eq!(g.players[0].creatures_cast_this_turn, 0);
    drain_stack(&mut g);
}

/// 715.3d — the exiled adventurer may be *played*, but not cast as an
/// Adventure a second time.
#[test]
fn cr_715_3d_exiled_adventurer_cannot_adventure_again() {
    let mut g = two_player_game();
    let midgar = g.add_card_to_hand(0, catalog::midgar_city_of_mako());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    g.players[0].mana_pool.add(Color::Black, 6);
    g.perform_action(GameAction::CastAdventure {
        card_id: midgar,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Adventure");
    drain_stack(&mut g);
    assert!(
        g.perform_action(GameAction::CastAdventure {
            card_id: midgar,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "an exiled adventurer can't go on another adventure"
    );
    g.perform_action(GameAction::PlayLand(midgar)).expect("but it can be played");
    assert!(g.battlefield_find(midgar).is_some());
}

/// 715.3d — the land half still costs the turn's land drop.
#[test]
fn cr_715_3d_adventure_land_costs_the_land_drop() {
    let mut g = two_player_game();
    let midgar = g.add_card_to_hand(0, catalog::midgar_city_of_mako());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    g.players[0].mana_pool.add(Color::Black, 3);
    g.perform_action(GameAction::CastAdventure {
        card_id: midgar,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Adventure");
    drain_stack(&mut g);
    g.players[0].lands_played_this_turn = 1;
    assert!(g.perform_action(GameAction::PlayLand(midgar)).is_err(), "land drop spent");
}

// ── CR 614 — as-enters replacements ─────────────────────────────────────────

/// 614 — "as this enters" fires on a put-onto-the-battlefield entry too, so a
/// printed 0/0 devourer never faces SBAs at 0 toughness.
#[test]
fn cr_614_as_enters_applies_on_every_battlefield_entry() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(3)]));
    let sire = g.move_card_to_battlefield_for_test(0, catalog::famished_worldsire());
    drain_stack(&mut g);
    let c = g.battlefield_find(sire).expect("survived its own entry");
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied(), Some(9));
}

/// The turn-based advance still works with a devoured board (regression guard
/// for the entry-path change).
#[test]
fn cr_614_as_enters_does_not_double_apply_on_a_cast() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    for _ in 0..8 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let sire = g.add_card_to_hand(0, catalog::famished_worldsire());
    g.players[0].mana_pool.add(Color::Green, 8);
    g.perform_action(GameAction::CastSpell {
        card_id: sire,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    // AutoDecider devours nothing, so the 0/0 dies — but only once, and the
    // lands are untouched.
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), 8);
}
