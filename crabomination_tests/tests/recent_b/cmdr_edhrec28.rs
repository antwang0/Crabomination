//! Commander: Samwise Gamgee's and Miles "Tails" Prower's last gaps
//! (`decks::cmdr_edhrec28`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
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

fn cast(g: &mut GameState, id: CardId) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

/// Campsite Cuisine makes a Food on entry and for each legendary creature.
#[test]
fn campsite_cuisine_feeds_on_legends() {
    let mut g = pod(2);
    let id = g.add_card_to_hand(0, catalog::campsite_cuisine());
    cast(&mut g, id).expect("cast");
    let legend = g.add_card_to_hand(0, catalog::kraven_the_hunter());
    cast(&mut g, legend).expect("legend");
    assert_eq!(named(&g, "Food"), 2);
}

/// The Ring Goes South puts one land per legendary creature onto the
/// battlefield tapped and bottoms the rest.
#[test]
fn the_ring_goes_south_finds_lands() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::kraven_the_hunter());
    ready(&mut g, 0, catalog::blor_the_impervious());
    for def in [catalog::grizzly_bears, catalog::forest, catalog::lightning_bolt, catalog::island, catalog::mountain] {
        g.add_card_to_library(0, def());
    }
    let id = g.add_card_to_hand(0, catalog::the_ring_goes_south());
    cast(&mut g, id).expect("cast");
    let lands: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).collect();
    assert_eq!(lands.len(), 2);
    assert!(lands.iter().all(|c| c.tapped));
    assert_eq!(g.players[0].library.len(), 3, "the bear and the bolt on the bottom, the Mountain unrevealed");
}

/// The Thanos-Copter animates your other Vehicles for the turn on entry.
#[test]
fn thanos_copter_animates_vehicles() {
    let mut g = pod(2);
    let other = ready(&mut g, 0, catalog::smugglers_copter());
    let creature = |g: &GameState| {
        g.computed_permanent(other).unwrap().card_types().contains(&crabomination::card::CardType::Creature)
    };
    assert!(!creature(&g));
    let copter = g.add_card_to_hand(0, catalog::the_thanos_copter());
    cast(&mut g, copter).expect("cast");
    assert!(creature(&g));
}

/// Lita makes a Zeppelin and untaps your other artifact creatures at end
/// step.
#[test]
fn lita_makes_zeppelins_and_untaps() {
    let mut g = pod(2);
    let lita = ready(&mut g, 0, catalog::lita_mechanical_engineer());
    let raven = ready(&mut g, 0, catalog::silver_raven());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lita,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("zeppelin");
    drain_stack(&mut g);
    let zep = g.battlefield.iter().find(|c| c.definition.name == "Zeppelin").expect("token");
    assert!(zep.definition.keywords.contains(&Keyword::Flying));
    g.battlefield_find_mut(raven).unwrap().tapped = true;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(raven).unwrap().tapped);
}
