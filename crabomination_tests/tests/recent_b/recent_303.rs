//! Tests for the recent303 Dissension batch 2 (Eidolon cycle + utility).

// (no card-type imports needed)
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, GameAction};
use crabomination::mana::Color;

#[test]
fn verdant_eidolon_sacs_for_three_mana() {
    let mut g = two_player_game();
    let e = g.add_card_to_battlefield(0, catalog::verdant_eidolon());
    g.clear_sickness(e);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: e, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("sac for mana");
    drain_stack(&mut g);
    assert!(g.battlefield_find(e).is_none(), "sacrificed as a cost");
    assert_eq!(g.players[0].mana_pool.total(), 3, "added three mana of one color");
}

#[test]
fn entropic_eidolon_drains_a_target() {
    let mut g = two_player_game();
    let e = g.add_card_to_battlefield(0, catalog::entropic_eidolon());
    g.clear_sickness(e);
    let (me, foe) = (g.players[0].life, g.players[1].life);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: e, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("sac to drain");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe - 1, "target lost 1");
    assert_eq!(g.players[0].life, me + 1, "you gained 1");
}

#[test]
fn eidolon_returns_itself_on_a_multicolored_cast() {
    let mut g = two_player_game();
    // Put an Eidolon in the graveyard, then cast a multicolored spell.
    let e = catalog::sandstorm_eidolon();
    let gid = g.add_card_to_graveyard(0, e);
    let multi = g.add_card_to_hand(0, catalog::boros_guildmage()); // {R/W}{R/W}
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    // The recur is a "you may" — accept it.
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: multi, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast multicolored");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == gid), "Eidolon recurred to hand");
}

#[test]
fn ragamuffyn_only_draws_while_hellbent() {
    let mut g = two_player_game();
    let rag = g.add_card_to_battlefield(0, catalog::ragamuffyn());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears()); // something to draw
    g.clear_sickness(rag);
    // A card in hand → the ability's Hellbent condition fails.
    let held = g.add_card_to_hand(0, catalog::grizzly_bears());
    let blocked = g.perform_action(GameAction::ActivateAbility {
        card_id: rag, ability_index: 0, target: None,
        additional_targets: vec![Target::Permanent(fodder)], x_value: None, mode: None,
    });
    assert!(blocked.is_err(), "can't activate with a card in hand");
    // Empty the hand and try again.
    g.players[0].hand.retain(|c| c.id != held);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rag, ability_index: 0, target: None,
        additional_targets: vec![Target::Permanent(fodder)], x_value: None, mode: None,
    }).expect("draws while hellbent");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the fodder");
    assert_eq!(g.players[0].hand.len(), 1, "drew a card");
}

#[test]
fn soulsworn_jury_counters_a_creature_spell() {
    let mut g = two_player_game();
    let jury = g.add_card_to_battlefield(0, catalog::soulsworn_jury());
    g.clear_sickness(jury);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear on stack");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: jury, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("counter the creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(jury).is_none(), "Jury sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "creature spell countered");
}

#[test]
fn demons_jester_grows_while_hellbent() {
    let mut g = two_player_game();
    let dj = g.add_card_to_battlefield(0, catalog::demons_jester());
    assert_eq!(g.computed_permanent(dj).unwrap().power, 4, "Hellbent +2/+1");
    g.add_card_to_hand(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(dj).unwrap().power, 2, "no pump with a card in hand");
}

#[test]
fn flame_kin_war_scout_immolates_the_next_creature() {
    let mut g = two_player_game();
    let scout = g.add_card_to_battlefield(0, catalog::flame_kin_war_scout());
    // Cast a creature so it enters through the real funnel and fires the watcher.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast the intruder");
    drain_stack(&mut g);
    assert!(g.battlefield_find(scout).is_none(), "scout sacrificed itself");
    assert!(g.battlefield_find(bear).is_none(), "the entering 2/2 took 4 and died");
}

#[test]
fn minister_of_impediments_taps_a_creature() {
    let mut g = two_player_game();
    let mi = g.add_card_to_battlefield(0, catalog::minister_of_impediments());
    g.clear_sickness(mi);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: mi, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("tap the creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "target tapped");
}
