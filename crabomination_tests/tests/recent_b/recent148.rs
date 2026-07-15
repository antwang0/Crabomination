//! Functionality tests for `catalog::sets::decks::recent148`.

use crabomination::catalog;
use crabomination::game::*;
use crabomination::game::two_player_game;

/// Faebloom Trick makes two Faerie flyers and taps an opponent's creature.
#[test]
fn faebloom_trick_tokens_and_tap() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::faebloom_trick());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(enemy)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let faeries = g.battlefield.iter()
        .filter(|c| c.definition.name == "Faerie" && c.controller == 0).count();
    assert_eq!(faeries, 2, "two Faerie tokens");
    assert!(g.battlefield_find(enemy).unwrap().tapped, "opponent's creature tapped");
}

/// Popular Egotist's sacrifice trigger drains an opponent for 1.
#[test]
fn popular_egotist_sacrifice_drains() {
    let mut g = two_player_game();
    let egotist = g.add_card_to_battlefield(0, catalog::popular_egotist());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: egotist,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate sac ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert!(
        g.computed_permanent(egotist).unwrap().keywords.contains(&crabomination::card::Keyword::Indestructible),
        "gained indestructible",
    );
    assert_eq!(g.players[1].life, opp_life - 1, "opponent drained for 1");
    assert_eq!(g.players[0].life, my_life + 1, "you gained 1");
}

/// Fear of Impostors counters a spell on ETB.
#[test]
fn fear_of_impostors_counters_on_etb() {
    let mut g = two_player_game();
    g.active_player_idx = 1; // p1's turn so they can cast at sorcery speed
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    // Flash in Fear of Impostors; its ETB counters the bear.
    let fear = g.add_card_to_hand(0, catalog::fear_of_impostors());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: fear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("flash the Nightmare");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear countered by ETB");
}

/// Overwhelmed Apprentice mills each opponent two on ETB.
#[test]
fn overwhelmed_apprentice_mills_opponents() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    g.move_card_to_battlefield_for_test(0, catalog::overwhelmed_apprentice());
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 2, "opponent milled two");
}

/// Cursed Windbreaker manifests a creature and equips it, granting flying.
#[test]
fn cursed_windbreaker_manifests_and_equips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let eq = g.move_card_to_battlefield_for_test(0, catalog::cursed_windbreaker());
    drain_stack(&mut g);
    // A 2/2 manifested creature exists and the Equipment is attached to it.
    let host = g.battlefield_find(eq).unwrap().attached_to.expect("equipped to the manifest");
    assert!(
        g.computed_permanent(host).unwrap().keywords.contains(&crabomination::card::Keyword::Flying),
        "equipped creature has flying",
    );
}
