//! Functionality tests for `catalog::sets::decks::tla` — TLA staples.

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

fn attack_with(g: &mut GameState, atk: CardId) {
    g.clear_sickness(atk);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(g);
}

/// Cat-Gator deals damage equal to the Swamps you control on ETB.
#[test]
fn cat_gator_pings_for_swamps() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::swamp());
    let cg = g.add_card_to_battlefield(0, catalog::cat_gator());
    let before = g.players[1].life;
    g.fire_self_etb_triggers(cg, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 2, "2 damage = 2 Swamps");
}

/// Cat-Owl untaps a target permanent when it attacks.
#[test]
fn cat_owl_untaps_on_attack() {
    let mut g = two_player_game();
    let owl = g.add_card_to_battlefield(0, catalog::cat_owl());
    let mana_rock = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mana_rock).unwrap().tapped = true;
    attack_with(&mut g, owl);
    assert!(!g.battlefield_find(mana_rock).unwrap().tapped, "target untapped");
}

/// Kyoshi Warriors makes a 1/1 Ally on ETB.
#[test]
fn kyoshi_warriors_makes_ally() {
    let mut g = two_player_game();
    let kw = g.add_card_to_battlefield(0, catalog::kyoshi_warriors());
    g.fire_self_etb_triggers(kw, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Ally"), 1);
}

/// The Walls of Ba Sing Se grant indestructible to your other permanents.
#[test]
fn walls_grant_indestructible() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::walls_of_ba_sing_se());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Indestructible),
        "other permanent gains indestructible"
    );
}

/// Wandering Musicians pump the team +1/+0 on attack.
#[test]
fn wandering_musicians_team_pump() {
    let mut g = two_player_game();
    let wm = g.add_card_to_battlefield(0, catalog::wandering_musicians());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    attack_with(&mut g, wm);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+0 → 3 power");
}

use crate::game::types::Target;

/// Stand player 0 at main phase with priority and plenty of mana.
fn ready0(g: &mut GameState) {
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..8 { g.players[0].mana_pool.add_colorless(1); }
    for c in [crate::mana::Color::White, crate::mana::Color::Blue, crate::mana::Color::Black,
              crate::mana::Color::Red, crate::mana::Color::Green] {
        g.players[0].mana_pool.add(c, 4);
    }
}

/// It'll Quench Ya! counters a spell whose controller can't pay {2}.
#[test]
fn itll_quench_ya_counters_unpaid() {
    let mut g = two_player_game();
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1; // p1's turn so they can cast a creature
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bears");
    let quench = g.add_card_to_hand(0, catalog::itll_quench_ya());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: quench, target: Some(Target::Permanent(bears)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast quench");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "bears countered (couldn't pay {{2}})");
}

/// Ozai's Cruelty deals 2 and forces two discards.
#[test]
fn ozais_cruelty_burns_and_discards() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let oc = g.add_card_to_hand(0, catalog::ozais_cruelty());
    ready0(&mut g);
    let life = g.players[1].life;
    let hand = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: oc, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "2 damage");
    assert_eq!(g.players[1].hand.len(), hand - 2, "discarded two");
}

/// Pillar Launch pumps, grants reach, and untaps.
#[test]
fn pillar_launch_pumps_and_untaps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let pl = g.add_card_to_hand(0, catalog::pillar_launch());
    ready0(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: pl, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords.contains(&Keyword::Reach));
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
}

/// Rocky Rebuke fights one-sided: your creature pings an opponent's.
#[test]
fn rocky_rebuke_one_sided_fight() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2 power
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2 toughness
    let rr = g.add_card_to_hand(0, catalog::rocky_rebuke());
    ready0(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: rr, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "their bear took 2 and died");
    assert!(g.battlefield_find(mine).is_some(), "ours is untouched (one-sided)");
}

/// Shared Roots fetches a basic land onto the battlefield tapped.
#[test]
fn shared_roots_ramps_a_basic() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let sr = g.add_card_to_hand(0, catalog::shared_roots());
    ready0(&mut g);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(forest)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: sr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let f = g.battlefield_find(forest).expect("forest on battlefield");
    assert!(f.tapped, "enters tapped");
}

/// United Front makes X Allies and counters the team.
#[test]
fn united_front_makes_allies_and_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let uf = g.add_card_to_hand(0, catalog::united_front());
    ready0(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: uf, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast X=2");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Ally"), 2, "made X=2 Allies");
    // The pre-existing bear got a +1/+1 counter (3/3).
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "team counter");
}

// ── Second wave ─────────────────────────────────────────────────────────────

use crate::mana::Color;

#[test]
fn water_tribe_captain_pumps_team() {
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::water_tribe_captain());
    g.clear_sickness(cap);
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cap, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate {5}");
    drain_stack(&mut g);
    let v = g.compute_battlefield();
    assert_eq!(v.iter().find(|c| c.id == ally).map(|c| (c.power, c.toughness)), Some((3, 3)));
}

#[test]
fn earth_kingdom_protectors_grants_indestructible() {
    let mut g = two_player_game();
    let prot = g.add_card_to_battlefield(0, catalog::earth_kingdom_protectors());
    g.clear_sickness(prot);
    let ally = g.add_card_to_battlefield(0, catalog::kyoshi_warriors());
    g.clear_sickness(ally);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: prot, ability_index: 0, target: Some(Target::Permanent(ally)),
        additional_targets: vec![], x_value: None,
    }).expect("sacrifice to grant indestructible");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == prot), "Protectors sacrificed");
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Indestructible));
}

#[test]
fn yip_yip_grants_flying_only_to_allies() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::kyoshi_warriors()); // is an Ally
    let yip = g.add_card_to_hand(0, catalog::yip_yip());
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: yip, target: Some(Target::Permanent(ally)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Yip Yip!");
    drain_stack(&mut g);
    let v = g.compute_battlefield();
    let a = v.iter().find(|c| c.id == ally).unwrap();
    assert_eq!((a.power, a.toughness), (5, 5), "+2/+2");
    assert!(a.keywords.contains(&Keyword::Flying), "Ally also gains flying");
}

#[test]
fn earth_kingdom_jailer_exiles_then_returns() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // MV 6 ≥ 3
    let jailer = g.add_card_to_hand(0, catalog::earth_kingdom_jailer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: jailer, target: Some(Target::Permanent(victim)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Jailer");
    drain_stack(&mut g);
    let jid = g.battlefield.iter().find(|c| c.definition.name == "Earth Kingdom Jailer").unwrap().id;
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "victim exiled");
    g.battlefield_find_mut(jid).unwrap().damage = 3; // lethal to the 3/3 Jailer
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == victim), "victim returns when Jailer leaves");
}

#[test]
fn first_time_flyer_grows_with_a_lesson_in_graveyard() {
    let mut g = two_player_game();
    let f = g.add_card_to_battlefield(0, catalog::first_time_flyer());
    // No Lesson yet → base 1/2.
    let v = g.compute_battlefield();
    assert_eq!(v.iter().find(|c| c.id == f).map(|c| (c.power, c.toughness)), Some((1, 2)));
    g.add_card_to_graveyard(0, catalog::yip_yip()); // a Lesson
    let v = g.compute_battlefield();
    assert_eq!(v.iter().find(|c| c.id == f).map(|c| (c.power, c.toughness)), Some((2, 3)));
}

#[test]
fn fire_nation_raider_clue_only_after_attacking() {
    let mut g = two_player_game();
    g.players[0].attacked_this_turn = true;
    let raider = g.add_card_to_battlefield(0, catalog::fire_nation_raider());
    g.fire_self_etb_triggers(raider, 0);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"), "Raid → Clue");
}

#[test]
fn wartime_protestors_buffs_incoming_allies() {
    let mut g = two_player_game();
    let prot = g.add_card_to_battlefield(0, catalog::wartime_protestors());
    g.clear_sickness(prot);
    // Cast an Ally so the "another Ally enters" watcher fires.
    let ally = g.add_card_to_hand(0, catalog::kyoshi_warriors());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: ally, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Kyoshi Warriors");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ally).unwrap().counters.get(&crate::card::CounterType::PlusOnePlusOne)
            .copied().unwrap_or(0),
        1,
        "incoming Ally gets a +1/+1 counter",
    );
}

#[test]
fn walltop_sentries_lifegain_needs_a_lesson() {
    let mut g = two_player_game();
    let life0 = g.players[0].life;
    let w = g.add_card_to_battlefield(0, catalog::walltop_sentries());
    g.add_card_to_graveyard(0, catalog::octopus_form()); // a Lesson
    g.battlefield_find_mut(w).unwrap().damage = 3; // lethal to the 2/3
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 2, "dies with a Lesson in gy → gain 2");
}

#[test]
fn saber_tooth_is_a_french_vanilla_reach() {
    let def = catalog::saber_tooth_moose_lion();
    assert_eq!((def.power, def.toughness), (7, 7));
    assert!(def.keywords.contains(&Keyword::Reach));
    assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Landcycling(_, _))));
}

#[test]
fn cunning_maneuver_pumps_and_clues() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cm = g.add_card_to_hand(0, catalog::cunning_maneuver());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: cm, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Cunning Maneuver");
    drain_stack(&mut g);
    let v = g.compute_battlefield();
    assert_eq!(v.iter().find(|c| c.id == bear).map(|c| (c.power, c.toughness)), Some((5, 3)));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"), "made a Clue");
}
