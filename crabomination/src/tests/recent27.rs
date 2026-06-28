//! Functionality tests for `catalog::sets::decks::recent27` — BLB/FIN/DSK
//! commons on existing primitives.

use crate::catalog;
use crate::card::Keyword;
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;
use crate::TurnStep;

fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == controller && c.definition.name == name)
        .count()
}

/// Brightblade Stoat is a 2/2 with first strike and lifelink.
#[test]
fn brightblade_stoat_keywords() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::brightblade_stoat());
    let cp = g.computed_permanent(s).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.keywords.contains(&Keyword::FirstStrike) && cp.keywords.contains(&Keyword::Lifelink));
}

/// Pond Prophet draws a card on entry.
#[test]
fn pond_prophet_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    let p = g.move_card_to_battlefield_for_test(0, catalog::pond_prophet());
    drain_stack(&mut g);
    let _ = p;
    assert_eq!(g.players[0].hand.len(), hand + 1, "Pond Prophet drew a card");
}

/// Hecteyes makes each opponent discard on entry.
#[test]
fn hecteyes_discards_each_opponent() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let opp_hand = g.players[1].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::hecteyes());
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent discarded a card");
}

/// Agate-Blade Assassin drains 1 on attack.
#[test]
fn agate_blade_assassin_drains_on_attack() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::agate_blade_assassin());
    g.battlefield_find_mut(a).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let (my, opp) = (g.players[0].life, g.players[1].life);
    g.declare_attackers(vec![Attack { attacker: a, target: AttackTarget::Player(1) }]).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "defender lost 1");
    assert_eq!(g.players[0].life, my + 1, "attacker gained 1");
}

/// Gigantoad is 4/4, pumping to 6/6 with seven lands.
#[test]
fn gigantoad_pumps_with_seven_lands() {
    let mut g = two_player_game();
    let toad = g.add_card_to_battlefield(0, catalog::gigantoad());
    assert_eq!(g.computed_permanent(toad).map(|c| (c.power, c.toughness)), Some((4, 4)));
    for _ in 0..7 { g.add_card_to_battlefield(0, catalog::forest()); }
    assert_eq!(g.computed_permanent(toad).map(|c| (c.power, c.toughness)), Some((6, 6)));
}

/// Loporrit Scout pumps itself when another creature enters.
#[test]
fn loporrit_scout_pumps_on_creature_etb() {
    let mut g = two_player_game();
    let scout = g.add_card_to_battlefield(0, catalog::loporrit_scout());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: other }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(scout).unwrap().power, 4, "scout pumped +1/+1");
}

/// Head of the Homestead makes two Rabbit tokens on entry.
#[test]
fn head_of_the_homestead_makes_rabbits() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::head_of_the_homestead());
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Rabbit"), 2, "two Rabbit tokens");
}

/// Dwarven Castle Guard mints a Hero token when it dies.
#[test]
fn dwarven_castle_guard_dies_to_hero() {
    let mut g = two_player_game();
    let guard = g.add_card_to_battlefield(0, catalog::dwarven_castle_guard());
    let mut evs = Vec::new();
    g.sacrifice_one(guard, 0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Hero"), 1, "made a Hero token on death");
}

/// Shrike Force is a 1/3 flyer with double strike and vigilance.
#[test]
fn shrike_force_keywords() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::shrike_force());
    let cp = g.computed_permanent(s).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying));
    assert!(cp.keywords.contains(&Keyword::DoubleStrike));
    assert!(cp.keywords.contains(&Keyword::Vigilance));
}

/// Moonrise Cleric gains 1 life when it attacks.
#[test]
fn moonrise_cleric_gains_life_on_attack() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::moonrise_cleric());
    g.battlefield_find_mut(c).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let my = g.players[0].life;
    g.declare_attackers(vec![Attack { attacker: c, target: AttackTarget::Player(1) }]).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, my + 1, "gained 1 life on attack");
}

/// Dragoon's Wyvern mints a Hero token on entry.
#[test]
fn dragoons_wyvern_makes_hero() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::dragoons_wyvern());
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Hero"), 1, "made a Hero token on entry");
}

/// Coeurl taps a target nonenchantment creature.
#[test]
fn coeurl_taps_target_creature() {
    let mut g = two_player_game();
    let coeurl = g.add_card_to_battlefield(0, catalog::coeurl());
    g.battlefield_find_mut(coeurl).unwrap().summoning_sick = false;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: coeurl, ability_index: 0,
        target: Some(crate::game::types::Target::Permanent(victim)), additional_targets: vec![], x_value: None,
    }).expect("activate Coeurl");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped, "target creature tapped");
}

/// Ahriman sacrifices another permanent to draw a card.
#[test]
fn ahriman_sacrifices_to_draw() {
    let mut g = two_player_game();
    let ahriman = g.add_card_to_battlefield(0, catalog::ahriman());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ahriman, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Ahriman");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card off the sacrifice");
}

/// Gaelicat gets +2/+0 with two artifacts.
#[test]
fn gaelicat_pumps_with_artifacts() {
    let mut g = two_player_game();
    let cat = g.add_card_to_battlefield(0, catalog::gaelicat());
    assert_eq!(g.computed_permanent(cat).unwrap().power, 1);
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    assert_eq!(g.computed_permanent(cat).unwrap().power, 3, "+2/+0 with two artifacts");
}

/// Scorpion Sentinel gets +3/+0 with seven lands.
#[test]
fn scorpion_sentinel_pumps_with_lands() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::scorpion_sentinel());
    assert_eq!(g.computed_permanent(s).unwrap().power, 1);
    for _ in 0..7 { g.add_card_to_battlefield(0, catalog::island()); }
    assert_eq!(g.computed_permanent(s).unwrap().power, 4, "1 +3 = 4 power");
}

/// Thistledown Players untaps a target nonland permanent on attack.
#[test]
fn thistledown_players_untaps_on_attack() {
    let mut g = two_player_game();
    let players = g.add_card_to_battlefield(0, catalog::thistledown_players());
    g.battlefield_find_mut(players).unwrap().summoning_sick = false;
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(ally).unwrap().tapped = true;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(crate::game::types::Target::Permanent(ally)),
    ]));
    g.declare_attackers(vec![Attack { attacker: players, target: AttackTarget::Player(1) }]).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield_find(ally).unwrap().tapped, "ally untapped by the attack trigger");
}

/// Warren Elder pumps the team +1/+1 until end of turn.
#[test]
fn warren_elder_team_pump() {
    let mut g = two_player_game();
    let elder = g.add_card_to_battlefield(0, catalog::warren_elder());
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: elder, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Warren Elder");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(buddy).unwrap().power, 3, "buddy pumped to 3");
    assert_eq!(g.computed_permanent(elder).unwrap().power, 3, "elder pumped to 3");
}
