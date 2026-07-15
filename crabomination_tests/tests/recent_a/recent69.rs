//! Functionality tests for `catalog::sets::decks::recent69`.

use crabomination::card::{Keyword, LandType};
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::*;

#[test]
fn frost_giant_has_rampage_2() {
    assert!(catalog::frost_giant().keywords.contains(&Keyword::Rampage(2)));
}

#[test]
fn highland_game_gains_life_on_death() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::highland_game());
    let before = g.players[0].life;
    let evs = g.remove_to_graveyard_with_triggers(id);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before + 2, "dies → gain 2 life");
}

#[test]
fn rushwood_dryad_has_forestwalk() {
    assert!(catalog::rushwood_dryad().keywords.contains(&Keyword::Landwalk(LandType::Forest)));
}

#[test]
fn ainok_tracker_first_strike_and_morph() {
    let d = catalog::ainok_tracker();
    assert!(d.keywords.contains(&Keyword::FirstStrike));
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Morph(_))));
}

#[test]
fn charging_slateback_cant_block_and_morph() {
    let d = catalog::charging_slateback();
    assert!(d.keywords.contains(&Keyword::CantBlock));
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Morph(_))));
}

#[test]
fn auriok_transfixer_taps_target_artifact() {
    let mut g = two_player_game();
    let tf = g.add_card_to_battlefield(0, catalog::auriok_transfixer());
    g.clear_sickness(tf);
    let art = g.add_card_to_battlefield(1, catalog::ornithopter());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tf, ability_index: 0, target: Some(Target::Permanent(art)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).unwrap().tapped, "target artifact tapped");
}

#[test]
fn snapping_creeper_gains_vigilance_on_landfall() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::snapping_creeper());
    assert!(!g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Vigilance));
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.dispatch_triggers_for_events(&[GameEvent::LandPlayed { player: 0, card_id: land }]);
    drain_stack(&mut g);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Vigilance),
        "landfall grants vigilance until end of turn");
}

#[test]
fn nyxborn_rollicker_bestows_plus_one() {
    let d = catalog::nyxborn_rollicker();
    assert!(d.bestow.is_some(), "has Bestow");
    let bonus = d.equipped_bonus.expect("bestow bonus");
    assert_eq!((bonus.power, bonus.toughness), (1, 1));
}
