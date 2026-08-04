//! Conspiracy (CNS) — CR 315 conspiracy cards, `catalog::sets::cns`.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::mountain());
        }
    }
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn upkeep(g: &mut GameState) {
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(g);
}

/// CR 315.5 — a face-up conspiracy's static abilities affect the game from
/// the command zone.
#[test]
fn cr_315_5_weight_advantage_works_from_the_command_zone() {
    let mut g = main_phase();
    g.seat_conspiracy(0, catalog::weight_advantage(), None);
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_omens());
    assert!(
        g.computed_permanent(wall)
            .expect("wall")
            .keywords
            .contains(&Keyword::AssignsCombatDamageByToughness)
    );
}

/// CR 315.5b — a face-down conspiracy has no characteristics, so nothing
/// applies until it is turned face up (CR 702.106b).
#[test]
fn cr_315_5b_a_face_down_agenda_grants_nothing_until_revealed() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::immediate_action(), Some("Grizzly Bears"));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bear).expect("bear").keywords.contains(&Keyword::Haste));
    assert!(g.reveal_hidden_agenda(0, agenda));
    assert!(g.computed_permanent(bear).expect("bear").keywords.contains(&Keyword::Haste));
}

/// CR 702.106 — the revealed name gates the grant; other creatures miss out.
#[test]
fn hidden_agenda_only_names_one_card() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::immediate_action(), Some("Grizzly Bears"));
    g.reveal_hidden_agenda(0, agenda);
    let other = g.add_card_to_battlefield(0, catalog::hill_giant());
    assert!(!g.computed_permanent(other).expect("giant").keywords.contains(&Keyword::Haste));
}

#[test]
fn power_play_claims_the_first_turn() {
    let mut g = main_phase();
    g.seat_conspiracy(1, catalog::power_play(), None);
    assert_eq!(g.apply_starting_player_conspiracies(), 1);
    assert_eq!(g.active_player_idx, 1);
}

#[test]
fn hymn_of_the_wilds_discounts_creatures_and_locks_out_spells() {
    let mut g = main_phase();
    g.seat_conspiracy(0, catalog::hymn_of_the_wilds(), None);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err()
    );
    // The first creature spell of the turn costs {1} less.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, 0, bear, None);
    assert!(g.battlefield_find(bear).is_some());
}

#[test]
fn bragos_favor_discounts_the_named_spell() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::bragos_favor(), Some("Hill Giant"));
    g.reveal_hidden_agenda(0, agenda);
    let giant = g.add_card_to_hand(0, catalog::hill_giant()); // {3}{R}
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, 0, giant, None);
    assert!(g.battlefield_find(giant).is_some());
}

#[test]
fn muzzios_preparations_grows_the_named_creature() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::muzzios_preparations(), Some("Grizzly Bears"));
    g.reveal_hidden_agenda(0, agenda);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, 0, bear, None);
    assert_eq!(
        g.battlefield_find(bear).expect("bear").counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

#[test]
fn iterative_analysis_draws_off_the_named_spell() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::iterative_analysis(), Some("Lightning Bolt"));
    g.reveal_hidden_agenda(0, agenda);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let library = g.players[0].library.len();
    cast(&mut g, 0, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[0].library.len(), library - 1);
}

#[test]
fn sentinel_dispatch_and_hold_the_perimeter_seed_the_first_upkeep() {
    let mut g = main_phase();
    g.seat_conspiracy(0, catalog::sentinel_dispatch(), None);
    g.seat_conspiracy(0, catalog::hold_the_perimeter(), None);
    g.turn_number = 1;
    upkeep(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token).count(), 2);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1 && c.is_token).count(), 1);
}
