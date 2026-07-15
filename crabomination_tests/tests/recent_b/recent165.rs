//! Functionality tests for `catalog::sets::decks::recent165` (Foundations).

use crabomination::catalog;
use crabomination::card::Keyword;
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::*;

/// Skyship Buccaneer's Raid draws when you attacked this turn.
#[test]
fn skyship_buccaneer_raid_draws() {
    let mut g = two_player_game();
    g.players[0].attacked_this_turn = true;
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    let bucc = g.add_card_to_battlefield(0, catalog::skyship_buccaneer());
    g.fire_self_etb_triggers(bucc, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "Raid drew a card");
}

/// Starlight Snare taps its target and locks it down.
#[test]
fn starlight_snare_taps_and_locks() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::starlight_snare());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Starlight Snare");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "ETB tapped the creature");
    // It won't untap while enchanted.
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(foe).unwrap().tapped, "stays tapped — the Aura locks its untap");
}

/// Inspiring Paladin has first strike only during its controller's turn.
#[test]
fn inspiring_paladin_first_strike_on_your_turn() {
    let mut g = two_player_game();
    let pal = g.add_card_to_battlefield(0, catalog::inspiring_paladin());
    g.active_player_idx = 0;
    assert!(g.computed_permanent(pal).unwrap().keywords.contains(&Keyword::FirstStrike), "first strike on your turn");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(pal).unwrap().keywords.contains(&Keyword::FirstStrike), "no first strike on the opponent's turn");
}

/// Dreadwing Scavenger loots on entry and gains deathtouch at Threshold.
#[test]
fn dreadwing_scavenger_loots_and_thresholds() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    let dread = g.add_card_to_battlefield(0, catalog::dreadwing_scavenger());
    // No Threshold yet → no deathtouch.
    assert!(!g.computed_permanent(dread).unwrap().keywords.contains(&Keyword::Deathtouch));
    g.fire_self_etb_triggers(dread, 0);
    drain_stack(&mut g);
    // Loot: drew then discarded → net hand unchanged (drew 1, discarded 1).
    assert_eq!(g.players[0].hand.len(), hand, "looted (draw then discard)");
    // Fill the graveyard to seven for Threshold.
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    assert!(g.computed_permanent(dread).unwrap().keywords.contains(&Keyword::Deathtouch), "Threshold grants deathtouch");
}
