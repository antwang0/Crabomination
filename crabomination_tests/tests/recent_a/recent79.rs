//! Functionality tests for `catalog::sets::decks::recent79`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

#[test]
fn carrion_ants_pump_stacks() {
    let mut g = two_player_game();
    let ants = g.add_card_to_battlefield(0, catalog::carrion_ants());
    g.players[0].mana_pool.add_colorless(2);
    for _ in 0..2 {
        g.perform_action(GameAction::ActivateAbility {
            card_id: ants, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("{1}: +1/+1");
        drain_stack(&mut g);
    }
    let p = g.computed_permanent(ants).unwrap();
    assert_eq!((p.power, p.toughness), (2, 3), "0/1 pumped twice");
}

#[test]
fn elvish_hunter_locks_untap() {
    let mut g = two_player_game();
    let hunter = g.add_card_to_battlefield(0, catalog::elvish_hunter());
    g.clear_sickness(hunter);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hunter, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("untap lock");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().skip_next_untap, "target flagged to skip its next untap");
}

#[test]
fn dwarven_nomad_makes_small_creature_unblockable() {
    let mut g = two_player_game();
    let nomad = g.add_card_to_battlefield(0, catalog::dwarven_nomad());
    g.clear_sickness(nomad);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    g.perform_action(GameAction::ActivateAbility {
        card_id: nomad, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("grant unblockable");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
}

#[test]
fn balduvian_war_makers_has_haste_and_rampage() {
    let c = catalog::balduvian_war_makers();
    assert!(c.keywords.contains(&Keyword::Haste));
    assert!(c.keywords.contains(&Keyword::Rampage(1)));
    assert_eq!((c.power, c.toughness), (3, 3));
}

#[test]
fn grave_robbers_exiles_artifact_and_gains_life() {
    let mut g = two_player_game();
    let robbers = g.add_card_to_battlefield(0, catalog::grave_robbers());
    g.clear_sickness(robbers);
    // An artifact in the opponent's graveyard.
    let art = g.add_card_to_battlefield(1, catalog::jayemdae_tome());
    g.remove_from_battlefield_to_graveyard_raw(art);
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: robbers, ability_index: 0, target: Some(Target::Permanent(art)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("exile artifact from graveyard");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == art), "artifact exiled from the graveyard");
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
}
