//! Functionality tests for `catalog::sets::decks::recent129` (WOE wave 2).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Moonshaker Cavalry gives the team flying and +X/+X for X creatures.
#[test]
fn moonshaker_cavalry_team_anthem() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cav = g.add_card_to_battlefield(0, catalog::moonshaker_cavalry());
    g.fire_self_etb_triggers(cav, 0);
    drain_stack(&mut g);
    // Two creatures → +2/+2 and flying.
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "bear 2/2 → 4/4");
    assert!(cp.keywords.contains(&Keyword::Flying), "and flying");
}

/// Water Wings makes your creature a 4/4 flier with hexproof.
#[test]
fn water_wings_transforms_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::water_wings());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Water Wings");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "base 4/4");
    assert!(cp.keywords.contains(&Keyword::Flying) && cp.keywords.contains(&Keyword::Hexproof));
}

/// Werefox Bodyguard exiles a creature until it leaves, and returns it on sac.
#[test]
fn werefox_bodyguard_exile_and_return() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let fox = g.add_card_to_battlefield(0, catalog::werefox_bodyguard());
    g.fire_self_etb_triggers(fox, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature exiled");
    // Sacrifice the Werefox to gain 2 and free the exile.
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fox,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("sac Werefox");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
    assert!(g.battlefield_find(victim).is_some(), "exiled creature returned when the Werefox left");
}

/// Grand Ball Guest grows and gains trample under Celebration.
#[test]
fn grand_ball_guest_celebration() {
    let mut g = two_player_game();
    let guest = g.add_card_to_battlefield(0, catalog::grand_ball_guest());
    assert_eq!(g.computed_permanent(guest).unwrap().power, 2, "no celebration → 2/2");
    g.players[0].nonland_permanents_entered_this_turn = 2;
    let cp = g.computed_permanent(guest).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "celebration → +1/+1");
    assert!(cp.keywords.contains(&Keyword::Trample), "and trample");
}

/// Ratcatcher Trainee's Pest Problem adventure makes two Rats.
#[test]
fn ratcatcher_pest_problem_makes_rats() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let card = g.add_card_to_hand(0, catalog::ratcatcher_trainee());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastAdventure {
        card_id: card,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Pest Problem");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Rat").count(),
        2,
        "two Rats made",
    );
}

/// Twisted Fealty steals a creature for the turn and drops a Wicked Role.
#[test]
fn twisted_fealty_steal_and_role() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let stolen = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::twisted_fealty());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(stolen)),
        additional_targets: vec![Target::Permanent(mine)],
        mode: None,
        x_value: None,
    })
    .expect("cast Twisted Fealty");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(stolen).unwrap().controller, 0, "gained control this turn");
    let cp = g.computed_permanent(mine).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "Wicked Role gives +1/+1");
}
