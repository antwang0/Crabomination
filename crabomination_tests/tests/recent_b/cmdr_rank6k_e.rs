//! Commander: two-card gaps of EDHREC ranks 5501-6500
//! (`decks::cmdr_rank6k_e`).

use crabomination::card::CardId;
use crabomination::catalog;
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .map(|c| c.id)
        .collect()
}

/// CR 602.2 / 107.3 — Mask of the Jadecrafter sacrifices itself for an X/X
/// Golem artifact creature token.
#[test]
fn mask_of_the_jadecrafter_makes_an_x_golem() {
    let mut g = pod(2);
    let mask = ready(&mut g, 0, catalog::mask_of_the_jadecrafter());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mask,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(4),
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let golem = named(&g, 0, "Golem");
    assert_eq!(golem.len(), 1);
    let c = g.computed_permanent(golem[0]).expect("golem");
    assert_eq!((c.power, c.toughness), (4, 4));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == mask), "the Mask was sacrificed");
}

/// CR 702.84a — Mask of the Jadecrafter unearths from the graveyard.
#[test]
fn mask_of_the_jadecrafter_unearths() {
    let mut g = pod(2);
    let mask = g.add_card_to_graveyard(0, catalog::mask_of_the_jadecrafter());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mask,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("unearth");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == mask));
}

/// CR 702.124 — The Prismatic Piper is a colorless 3/3 partner.
#[test]
fn the_prismatic_piper_is_a_colorless_partner() {
    let d = catalog::the_prismatic_piper();
    assert!(d.keywords.contains(&crabomination::card::Keyword::Partner));
    assert!(d.printed_colors().is_empty());
    assert_eq!((d.power, d.toughness), (3, 3));
}

/// CR 702.16b / 702.16e — Rebbec gives each artifact you control protection
/// from every mana value among your artifacts: an opponent's {1}{R} Shatter
/// (MV 2) can't target your Sol Ring while you control MV-2 Mind Stone, and
/// a MV-2 source's damage to it is prevented while a MV-4 one's isn't.
#[test]
fn rebbec_protects_artifacts_from_their_mana_values() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::rebbec_architect_of_ascension());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    ready(&mut g, 0, catalog::mind_stone());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    assert!(g.damage_prevented_by_protection(bears, ring), "MV 2 matches Mind Stone");
    assert!(!g.damage_prevented_by_protection(giant, ring), "no artifact of MV 4");

    let shatter = g.add_card_to_hand(1, catalog::shatter());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    flood(&mut g, 1);
    let r = g.perform_action(GameAction::CastSpell {
        card_id: shatter,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(r.is_err(), "Shatter can't target the protected Sol Ring");
}
