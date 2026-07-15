//! Functionality tests for `catalog::sets::decks::recent43` — utility-land
//! charge bombs, land destruction, graveyard recursion, and devotion mana.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::*;

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: idx, target, additional_targets: Vec::new(), x_value: None,
    }).expect("ability activates");
    drain_stack(g);
}

#[test]
fn blast_zone_charges_then_detonates() {
    let mut g = two_player_game();
    let bz = g.add_card_to_battlefield(0, catalog::blast_zone());
    // Pump to two charge counters via the {X}{X} ability (X=2 → costs 4).
    g.battlefield_find_mut(bz).unwrap().add_counters(CounterType::Charge, 1); // simulate ETB charge
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bz, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: Some(1),
    }).expect("charge");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bz).unwrap().counter_count(CounterType::Charge), 2,
        "1 (ETB) + 1 (X=1) charge counters"
    );
    // Untap (the charge ability tapped it) and detonate (MV 2): a {1}{G} bear
    // dies, a 1-drop lives.
    g.battlefield_find_mut(bz).unwrap().tapped = false;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, bz, 2, None);
    assert!(g.battlefield_find(bear).is_none(), "MV-2 permanent destroyed");
    assert!(g.battlefield_find(elf).is_some(), "MV-1 permanent spared");
}

#[test]
fn encroaching_wastes_destroys_a_nonbasic_land() {
    let mut g = two_player_game();
    let edge = g.add_card_to_battlefield(0, catalog::encroaching_wastes());
    let target = g.add_card_to_battlefield(1, catalog::wasteland()); // nonbasic
    g.players[0].mana_pool.add_colorless(4);
    activate(&mut g, edge, 1, Some(Target::Permanent(target)));
    assert!(g.battlefield_find(target).is_none(), "nonbasic land destroyed");
    assert!(g.battlefield_find(edge).is_none(), "Encroaching Wastes sacrificed itself");
}

#[test]
fn tectonic_edge_needs_an_opponent_with_four_lands() {
    let mut g = two_player_game();
    let edge = g.add_card_to_battlefield(0, catalog::tectonic_edge());
    let victim = g.add_card_to_battlefield(1, catalog::wasteland());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // Opponent controls only one land → activation illegal.
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: edge, ability_index: 1, target: Some(Target::Permanent(victim)),
        additional_targets: Vec::new(), x_value: None,
    });
    assert!(res.is_err(), "can't blow up a land until the opponent has four");
    // Give them three more lands (four total) → now legal.
    for _ in 0..3 { g.add_card_to_battlefield(1, catalog::island()); }
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, edge, 1, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none(), "now destroyed");
}

#[test]
fn buried_ruin_returns_an_artifact_from_the_graveyard() {
    let mut g = two_player_game();
    let ruin = g.add_card_to_battlefield(0, catalog::buried_ruin());
    let art = g.add_card_to_graveyard(0, catalog::ratchet_bomb());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, ruin, 1, Some(Target::Permanent(art)));
    assert!(g.players[0].hand.iter().any(|c| c.id == art), "artifact returned to hand");
}

#[test]
fn serras_sanctum_scales_with_enchantments() {
    let mut g = two_player_game();
    let sanctum = g.add_card_to_battlefield(0, catalog::serras_sanctum());
    g.add_card_to_battlefield(0, catalog::mark_of_asylum()); // enchantment
    g.add_card_to_battlefield(0, catalog::glacial_chasm()); // a land, not counted
    activate(&mut g, sanctum, 0, None);
    assert_eq!(g.players[0].mana_pool.total(), 1, "one W per enchantment (one enchantment)");
}

#[test]
fn tolarian_academy_scales_with_artifacts() {
    let mut g = two_player_game();
    let academy = g.add_card_to_battlefield(0, catalog::tolarian_academy());
    g.add_card_to_battlefield(0, catalog::ratchet_bomb());
    g.add_card_to_battlefield(0, catalog::sphere_of_the_suns());
    activate(&mut g, academy, 0, None);
    assert_eq!(g.players[0].mana_pool.total(), 2, "one U per artifact (two artifacts)");
}
