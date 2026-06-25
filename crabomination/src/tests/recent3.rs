//! Functionality tests for the `catalog::sets::decks::recent3` batch.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::types::{Attack, AttackTarget, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Solphim doubles noncombat damage a source you control deals to an opponent.
#[test]
fn solphim_doubles_noncombat_damage_to_opponent() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::solphim_mayhem_dominus());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt()); // 3 to any target
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "3 damage doubled to 6");
}

/// Solphim does NOT double combat damage (noncombat-only rider).
#[test]
fn solphim_leaves_combat_damage_alone() {
    let mut g = two_player_game();
    let solphim = g.add_card_to_battlefield(0, catalog::solphim_mayhem_dominus()); // 5/4
    g.clear_sickness(solphim);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: solphim, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 15, "5 combat damage, not doubled");
}

/// Atraxa ships flying/vigilance/deathtouch/lifelink and proliferates at end step.
#[test]
fn atraxa_proliferates_at_end_step() {
    let a = catalog::atraxa_praetors_voice();
    for kw in [Keyword::Flying, Keyword::Vigilance, Keyword::Deathtouch, Keyword::Lifelink] {
        assert!(a.keywords.contains(&kw), "Atraxa has {kw:?}");
    }
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::atraxa_praetors_voice());
    // A creature with a +1/+1 counter to proliferate onto.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "proliferate added a +1/+1 counter"
    );
}

/// Deathrite Shaman's first ability exiles a land from a graveyard for mana.
#[test]
fn deathrite_exiles_land_for_mana() {
    let mut g = two_player_game();
    let shaman = g.add_card_to_battlefield(0, catalog::deathrite_shaman());
    g.clear_sickness(shaman);
    let land = g.add_card_to_graveyard(1, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: shaman, ability_index: 0,
        target: Some(Target::Permanent(land)), additional_targets: vec![], x_value: None,
    }).expect("activate land exile");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == land), "land exiled");
    assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
}

/// Deathrite's instant/sorcery ability drains each opponent for 2.
#[test]
fn deathrite_drains_on_instant_exile() {
    let mut g = two_player_game();
    let shaman = g.add_card_to_battlefield(0, catalog::deathrite_shaman());
    g.clear_sickness(shaman);
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: shaman, ability_index: 1,
        target: Some(Target::Permanent(bolt)), additional_targets: vec![], x_value: None,
    }).expect("activate I/S exile");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "instant exiled");
    assert_eq!(g.players[1].life, 18, "opponent drained 2");
}

/// Grand Abolisher stops opponents casting + activating A/C/E abilities on your turn.
#[test]
fn grand_abolisher_locks_opponent_on_your_turn() {
    let mut g = two_player_game(); // P0 active
    g.add_card_to_battlefield(0, catalog::grand_abolisher());
    // P1 holds a spell and has priority during P0's turn.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "opponent can't cast during your turn");
    // A creature activated ability is also locked.
    let dork = g.add_card_to_battlefield(1, catalog::deathrite_shaman());
    g.clear_sickness(dork);
    let land = g.add_card_to_graveyard(0, catalog::forest());
    let err2 = g.perform_action(GameAction::ActivateAbility {
        card_id: dork, ability_index: 0,
        target: Some(Target::Permanent(land)), additional_targets: vec![], x_value: None,
    });
    assert!(err2.is_err(), "opponent can't activate creature abilities on your turn");
}

/// Sundering Titan destroys a land of each basic type on enter.
#[test]
fn sundering_titan_destroys_one_of_each_basic_type() {
    let mut g = two_player_game();
    let plains = g.add_card_to_battlefield(1, catalog::plains());
    let island = g.add_card_to_battlefield(1, catalog::island());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.move_card_to_battlefield_for_test(0, catalog::sundering_titan());
    drain_stack(&mut g);
    let in_a_graveyard = |g: &GameState, id| {
        g.players.iter().any(|p| p.graveyard.iter().any(|c| c.id == id))
    };
    for (id, name) in [(plains, "Plains"), (island, "Island"), (forest, "Forest")] {
        assert!(in_a_graveyard(&g, id), "{name} destroyed");
    }
}
