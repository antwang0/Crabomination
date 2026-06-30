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

#[test]
fn earth_kingdom_soldier_distributes_two_counters() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sol = g.add_card_to_battlefield(0, catalog::earth_kingdom_soldier());
    g.fire_self_etb_triggers(sol, 0); // up to two of my creatures auto-targeted
    drain_stack(&mut g);
    let total: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counters.get(&crate::card::CounterType::PlusOnePlusOne).copied().unwrap_or(0))
        .sum();
    assert_eq!(total, 2, "two +1/+1 counters distributed");
}

#[test]
fn white_lotus_anthems_other_allies() {
    let mut g = two_player_game();
    let lotus = g.add_card_to_battlefield(0, catalog::white_lotus_reinforcements());
    let ally = g.add_card_to_battlefield(0, catalog::kyoshi_warriors()); // an Ally
    let nonally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let v = g.compute_battlefield();
    assert_eq!(v.iter().find(|c| c.id == ally).map(|c| (c.power, c.toughness)), Some((4, 4)), "ally buffed");
    assert_eq!(v.iter().find(|c| c.id == nonally).map(|c| (c.power, c.toughness)), Some((2, 2)), "non-Ally not buffed");
    assert_eq!(v.iter().find(|c| c.id == lotus).map(|c| (c.power, c.toughness)), Some((2, 3)), "not self");
}

#[test]
fn combustion_technique_scales_with_lessons() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5
    g.add_card_to_graveyard(0, catalog::octopus_form()); // Lesson #1
    g.add_card_to_graveyard(0, catalog::yip_yip());      // Lesson #2
    let ct = g.add_card_to_hand(0, catalog::combustion_technique());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: ct, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Combustion Technique");
    drain_stack(&mut g);
    // 2 + 2 Lessons = 4 damage to the 5/5 (survives with 4 marked).
    assert_eq!(g.battlefield_find(foe).map(|c| c.damage), Some(4));
}

/// Iroh's Demonstration's default mode pings each opponent creature for 1.
#[test]
fn irohs_demonstration_sweep_mode() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::irohs_demonstration());
    ready0(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast (default sweep mode)");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).map(|c| c.damage), Some(1));
    assert_eq!(g.battlefield_find(b).map(|c| c.damage), Some(1));
}

/// Iroh's Demonstration mode 1 deals 4 to a single target creature.
#[test]
fn irohs_demonstration_burn_mode() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5
    let id = g.add_card_to_hand(0, catalog::irohs_demonstration());
    ready0(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![1])]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast (burn mode)");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(foe).map(|c| c.damage), Some(4));
}

/// Azula Always Lies runs both modes — a -1/-1 and a +1/+1 counter, each on its
/// own target.
#[test]
fn azula_always_lies_both_modes() {
    let mut g = two_player_game();
    let shrink = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let grow = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::azula_always_lies());
    ready0(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(shrink)),
        additional_targets: vec![Target::Permanent(grow)], mode: None, x_value: None,
    }).expect("cast (both modes)");
    drain_stack(&mut g);
    let s = g.computed_permanent(shrink).unwrap();
    assert_eq!((s.power, s.toughness), (1, 1), "-1/-1");
    let gv = g.computed_permanent(grow).unwrap();
    assert_eq!((gv.power, gv.toughness), (3, 3), "+1/+1 counter");
}

/// Tiger-Dillo can't attack alone, but can once another power-4 creature joins.
#[test]
fn tiger_dillo_gated_on_power_four() {
    let mut g = two_player_game();
    let td = g.add_card_to_battlefield(0, catalog::tiger_dillo());
    g.clear_sickness(td);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: td, target: AttackTarget::Player(1),
    }])).is_err(), "self doesn't satisfy 'another' power-4 creature");
    let helper = g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
    g.clear_sickness(helper);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: td, target: AttackTarget::Player(1),
    }])).expect("now a power-4 ally is present");
}

/// Raucous Audience taps for {G}, or {G}{G} with a power-4 creature out.
#[test]
fn raucous_audience_conditional_mana() {
    let mut g = two_player_game();
    let ra = g.add_card_to_battlefield(0, catalog::raucous_audience());
    g.clear_sickness(ra);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ra, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "just {{G}}");
    // Untap, add a power-4 creature, tap again → {G}{G}.
    g.battlefield_find_mut(ra).unwrap().tapped = false;
    g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
    g.perform_action(GameAction::ActivateAbility {
        card_id: ra, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana again");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3, "1 + 2 = 3 green");
}

/// Great Divide Guide grants "{T}: Add any color" to your lands and Allies.
#[test]
fn great_divide_guide_grants_mana_ability() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::great_divide_guide());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let ally = g.add_card_to_battlefield(0, catalog::kyoshi_warriors()); // an Ally
    let nonally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Lands and Allies pick up an extra tap-for-any-color ability.
    assert!(!g.granted_abilities_for(land).is_empty(), "land granted a mana ability");
    assert!(!g.granted_abilities_for(ally).is_empty(), "Ally granted a mana ability");
    assert!(g.granted_abilities_for(nonally).is_empty(), "non-Ally creature unchanged");
}

/// Gather the White Lotus mints one Ally per Plains.
#[test]
fn gather_white_lotus_scales_with_plains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::plains());
    g.add_card_to_battlefield(0, catalog::plains());
    g.add_card_to_battlefield(0, catalog::plains());
    let gw = g.add_card_to_hand(0, catalog::gather_the_white_lotus());
    ready0(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: gw, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Ally"), 3, "three Plains → three Allies");
}

/// Momo's leave-the-battlefield trigger (default mode) makes a Food.
#[test]
fn momo_playful_pet_ltb_makes_food() {
    let mut g = two_player_game();
    let m = g.add_card_to_battlefield(0, catalog::momo_playful_pet());
    // Sacrifice/destroy it to trigger the LTB.
    g.remove_to_graveyard_with_triggers(m);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"), "Food minted on LTB");
}

/// Tiger-Seal untaps when you draw your second card each turn.
#[test]
fn tiger_seal_untaps_on_second_draw() {
    let mut g = two_player_game();
    let ts = g.add_card_to_battlefield(0, catalog::tiger_seal());
    g.battlefield_find_mut(ts).unwrap().tapped = true;
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    g.players[0].cards_drawn_this_turn = 0;
    let mut ev = vec![];
    g.draw_one(0, &mut ev); // first
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ts).unwrap().tapped, "still tapped after one draw");
    let mut ev2 = vec![];
    g.draw_one(0, &mut ev2); // second
    g.dispatch_triggers_for_events(&ev2);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(ts).unwrap().tapped, "untapped on second draw");
}

/// The Spirit Oasis draws one card per Shrine you control.
#[test]
fn spirit_oasis_draws_per_shrine() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::northern_air_temple()); // another Shrine
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let hand = g.players[0].hand.len();
    let so = g.add_card_to_battlefield(0, catalog::the_spirit_oasis());
    g.fire_self_etb_triggers(so, 0);
    drain_stack(&mut g);
    // Two Shrines on the battlefield → draw 2.
    assert_eq!(g.players[0].hand.len(), hand + 2, "draws per Shrine");
}

/// Northern Air Temple drains X = number of Shrines on ETB.
#[test]
fn northern_air_temple_drains_per_shrine() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_spirit_oasis()); // another Shrine
    let life = g.players[1].life;
    let nat = g.add_card_to_battlefield(0, catalog::northern_air_temple());
    g.fire_self_etb_triggers(nat, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "two Shrines → drain 2");
}

/// Epic Downfall exiles a creature with mana value 3 or greater.
#[test]
fn epic_downfall_exiles_big_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // MV 6
    let ed = g.add_card_to_hand(0, catalog::epic_downfall());
    ready0(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: ed, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "exiled");
    assert!(g.exile.iter().any(|c| c.id == foe), "in exile, not graveyard");
}

/// Callous Inspector pings its controller and investigates on death.
#[test]
fn callous_inspector_dies_pings_and_clues() {
    let mut g = two_player_game();
    let ci = g.add_card_to_battlefield(0, catalog::callous_inspector());
    let life = g.players[0].life;
    g.remove_to_graveyard_with_triggers(ci);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "1 damage to you");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"), "made a Clue");
}

/// Canyon Crawler makes a Food on ETB.
#[test]
fn canyon_crawler_makes_food() {
    let mut g = two_player_game();
    let cc = g.add_card_to_battlefield(0, catalog::canyon_crawler());
    g.fire_self_etb_triggers(cc, 0);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"));
}

/// Foggy Swamp Hunters gains lifelink and menace once you've drawn two cards.
#[test]
fn foggy_swamp_hunters_keywords_after_two_draws() {
    let mut g = two_player_game();
    let f = g.add_card_to_battlefield(0, catalog::foggy_swamp_hunters());
    g.players[0].cards_drawn_this_turn = 1;
    let kws = g.computed_permanent(f).unwrap().keywords;
    assert!(!kws.contains(&Keyword::Lifelink) && !kws.contains(&Keyword::Menace), "off at 1 draw");
    g.players[0].cards_drawn_this_turn = 2;
    let kws = g.computed_permanent(f).unwrap().keywords;
    assert!(kws.contains(&Keyword::Lifelink) && kws.contains(&Keyword::Menace), "on at 2 draws");
}

/// June is unblockable once you've drawn two cards this turn.
#[test]
fn june_unblockable_after_two_draws() {
    let mut g = two_player_game();
    let j = g.add_card_to_battlefield(0, catalog::june_bounty_hunter());
    g.players[0].cards_drawn_this_turn = 2;
    assert!(g.computed_permanent(j).unwrap().keywords.contains(&Keyword::Unblockable));
}

/// Fire Sages grows with its +1/+1 counter ability.
#[test]
fn fire_sages_grows_with_counter() {
    let mut g = two_player_game();
    let fs = g.add_card_to_battlefield(0, catalog::fire_sages());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: fs, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(fs).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "2/2 + counter = 3/3");
}

/// Earth King's Lieutenant counters every other Ally on ETB.
#[test]
fn earth_kings_lieutenant_counters_allies() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::kyoshi_warriors()); // an Ally, 3/3
    let ekl = g.add_card_to_battlefield(0, catalog::earth_kings_lieutenant());
    g.fire_self_etb_triggers(ekl, 0);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ally).unwrap().power, 4, "+1/+1 on the other Ally");
}

/// Sandbenders' Storm's default mode destroys a power-4 creature.
#[test]
fn sandbenders_storm_destroys_big_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5
    let ss = g.add_card_to_hand(0, catalog::sandbenders_storm());
    ready0(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: ss, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "destroyed");
}

/// Azula, On the Hunt loses 1 life and investigates on attack.
#[test]
fn azula_on_the_hunt_attack_drains_and_clues() {
    let mut g = two_player_game();
    let az = g.add_card_to_battlefield(0, catalog::azula_on_the_hunt());
    let life = g.players[0].life;
    attack_with(&mut g, az);
    assert_eq!(g.players[0].life, life - 1, "lost 1 life");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"), "made a Clue");
}

/// Rabaroo Troop gains flying and a life on landfall.
#[test]
fn rabaroo_troop_landfall_flies() {
    let mut g = two_player_game();
    let rt = g.add_card_to_battlefield(0, catalog::rabaroo_troop());
    let land = g.add_card_to_hand(0, catalog::forest());
    ready0(&mut g);
    let life = g.players[0].life;
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert!(g.computed_permanent(rt).unwrap().keywords.contains(&Keyword::Flying), "gained flying");
    assert_eq!(g.players[0].life, life + 1, "gained 1 life");
}

/// Day of Black Sun (X=2) strips and destroys creatures with MV ≤ 2 only.
#[test]
fn day_of_black_sun_destroys_small_creatures() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let big = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // MV 6
    let dbs = g.add_card_to_hand(0, catalog::day_of_black_sun());
    ready0(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: dbs, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast with X=2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_none(), "MV-2 destroyed");
    assert!(g.battlefield_find(big).is_some(), "MV-6 survives");
}

/// Master Piandao digs an Ally from the top four into hand on attack.
#[test]
fn master_piandao_digs_an_ally() {
    let mut g = two_player_game();
    let mp = g.add_card_to_battlefield(0, catalog::master_piandao());
    // Top of library: an Ally, then non-matches.
    let ally = g.add_card_to_library(0, catalog::kyoshi_warriors());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    attack_with(&mut g, mp);
    assert!(g.players[0].hand.iter().any(|c| c.id == ally), "Ally pulled to hand");
}

/// Beetle-Headed Merchants sacrifices on attack to draw and grow.
#[test]
fn beetle_headed_merchants_sacrifices_to_draw_and_grow() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let beetle = g.add_card_to_battlefield(0, catalog::beetle_headed_merchants());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack_with(&mut g, beetle);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert_eq!(g.computed_permanent(beetle).unwrap().power, 6, "grew to 6 power");
}

/// Lo and Li grant lifelink to your Nobles and fetch a Lesson/Noble.
#[test]
fn lo_and_li_anthem_and_tutor() {
    let mut g = two_player_game();
    // A Noble already on board picks up lifelink.
    let noble = g.add_card_to_battlefield(0, catalog::azula_on_the_hunt()); // Human Noble
    let lesson = g.add_card_to_library(0, catalog::combustion_technique()); // a Lesson
    let ll = g.add_card_to_battlefield(0, catalog::lo_and_li_twin_tutors());
    assert!(g.computed_permanent(noble).unwrap().keywords.contains(&Keyword::Lifelink), "Noble lifelink");
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(lesson)),
    ]));
    g.fire_self_etb_triggers(ll, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == lesson), "tutored the Lesson to hand");
}

/// Fire Navy Trebuchet mints a tapped, attacking Ballistic Boulder when you
/// attack.
#[test]
fn fire_navy_trebuchet_makes_attacking_boulder() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fire_navy_trebuchet());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack_with(&mut g, bear); // attacking with the bear fires "whenever you attack"
    let boulder = g.battlefield.iter().find(|c| c.definition.name == "Ballistic Boulder");
    assert!(boulder.is_some(), "Ballistic Boulder minted");
    let b = boulder.unwrap();
    assert!(b.tapped, "enters tapped");
    assert!(g.attacking.iter().any(|a| a.attacker == b.id), "and attacking");
}

/// Hog-Monkey gives menace to a counter-bearing creature at combat.
#[test]
fn hog_monkey_grants_menace_to_counter_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hog_monkey());
    let buff = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(buff).unwrap().add_counters(crate::card::CounterType::PlusOnePlusOne, 1);
    g.clear_sickness(buff);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert!(g.computed_permanent(buff).unwrap().keywords.contains(&Keyword::Menace), "got menace at combat");
}

/// `legal_attackers` excludes Tiger-Dillo when it's the only power-4 creature
/// (the "another" gate, exclude_self) — keeps the bot/UI from offering an
/// illegal swing.
#[test]
fn tiger_dillo_not_a_legal_attacker_alone() {
    let mut g = two_player_game();
    let td = g.add_card_to_battlefield(0, catalog::tiger_dillo());
    g.clear_sickness(td);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    assert!(!g.legal_attackers(0).contains(&td), "not legal alone");
    let helper = g.add_card_to_battlefield(0, catalog::shivan_dragon()); // another power-4
    g.clear_sickness(helper);
    assert!(g.legal_attackers(0).contains(&td), "legal once a power-4 ally is present");
}

/// Energybending grants every basic land type to your lands and draws.
#[test]
fn energybending_fixes_lands_and_draws() {
    use crate::card::LandType;
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let eb = g.add_card_to_hand(0, catalog::energybending());
    ready0(&mut g);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: eb, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let lts = g.computed_permanent(forest).unwrap().subtypes.land_types;
    for lt in [LandType::Plains, LandType::Island, LandType::Swamp, LandType::Mountain, LandType::Forest] {
        assert!(lts.contains(&lt), "Forest gained {lt:?}");
    }
    // The spell left hand (−1) and we drew (+1) → net unchanged.
    assert_eq!(g.players[0].hand.len(), hand, "discard-to-stack then draw");
}

/// Swampsnare Trap saps -5/-3 from the enchanted creature.
#[test]
fn swampsnare_trap_weakens_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5
    let trap = g.add_card_to_hand(0, catalog::swampsnare_trap());
    ready0(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: trap, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    let cp = g.computed_permanent(foe).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 2), "5/5 → 0/2");
}

/// Flopsie counters your team on ETB and shields your big creatures from gang
/// blocks.
#[test]
fn flopsie_counters_and_shields_big_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/3
    let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5 power-4+
    let fl = g.add_card_to_battlefield(0, catalog::flopsie_bumis_buddy());
    g.fire_self_etb_triggers(fl, 0);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+1 counter");
    assert!(
        g.computed_permanent(dragon).unwrap().keywords.contains(&Keyword::CantBeBlockedByMoreThanOne),
        "power-4+ creature can't be gang-blocked"
    );
    assert!(
        !g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::CantBeBlockedByMoreThanOne),
        "the 3/3 (counter'd from 2/2) is power 3 — not shielded"
    );
}

/// Professor Zei returns an instant or sorcery from the graveyard, sacrificing
/// himself.
#[test]
fn professor_zei_returns_instant_from_graveyard() {
    let mut g = two_player_game();
    let zei = g.add_card_to_battlefield(0, catalog::professor_zei_anthropologist());
    g.clear_sickness(zei);
    let bolt = g.add_card_to_graveyard(0, catalog::pillar_launch()); // an instant
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: zei, ability_index: 1, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], x_value: None,
    }).expect("return I/S");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "instant back in hand");
    assert!(g.battlefield_find(zei).is_none(), "Zei sacrificed");
}

/// Foggy Swamp Spirit Keeper makes a Spirit on your second draw each turn.
#[test]
fn foggy_swamp_spirit_keeper_makes_spirit_on_second_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::foggy_swamp_spirit_keeper());
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    g.players[0].cards_drawn_this_turn = 0;
    let mut ev = vec![];
    g.draw_one(0, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Spirit"), 0, "none after one draw");
    let mut ev2 = vec![];
    g.draw_one(0, &mut ev2);
    g.dispatch_triggers_for_events(&ev2);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Spirit"), 1, "Spirit on second draw");
}

/// The TLA sac-lands tap for either color and can be cracked for a card.
#[test]
fn tla_sac_land_taps_and_cracks() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::north_pole_gates());
    g.add_card_to_library(0, catalog::forest());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // Mana ability 0 → white.
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for W");
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
    // Untap, then crack it: {4}, {T}, Sacrifice → draw.
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(4);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: vec![], x_value: None,
    }).expect("crack for a card");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Gran-Gran reduces noncreature spells by {1} only with 3+ Lessons in the
/// graveyard, and never reduces a creature spell.
#[test]
fn gran_gran_lesson_gated_cost_reduction() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gran_gran());
    let noncreature = crate::card::CardInstance::new(g.next_id(), catalog::boomerang_basics(), 0);
    let creature = crate::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);

    // Two Lessons → no reduction yet.
    g.add_card_to_graveyard(0, catalog::boomerang_basics());
    g.add_card_to_graveyard(0, catalog::yip_yip());
    assert_eq!(cost_reduction_for_spell(&g, 0, &noncreature, None), 0, "2 Lessons → no discount");

    // Third Lesson → {1} off the noncreature spell, still nothing off a creature.
    g.add_card_to_graveyard(0, catalog::fancy_footwork());
    assert_eq!(cost_reduction_for_spell(&g, 0, &noncreature, None), 1, "3 Lessons → {{1}} off");
    assert_eq!(cost_reduction_for_spell(&g, 0, &creature, None), 0, "never reduces creatures");
}

/// South Pole Voyager gains 1 life per Ally ETB and draws only on the second
/// resolution this turn.
#[test]
fn south_pole_voyager_draws_on_second_ally() {
    let mut g = two_player_game();
    let spv = g.add_card_to_battlefield(0, catalog::south_pole_voyager());
    g.clear_sickness(spv);
    g.add_card_to_library(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life0 = g.players[0].life;

    // One Kyoshi Warriors → it enters (Ally) and mints an Ally token: two ETBs.
    let ally = g.add_card_to_hand(0, catalog::kyoshi_warriors());
    let hand0 = g.players[0].hand.len(); // counts the Ally still in hand
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: ally, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ally");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life0 + 2, "1 life per Ally ETB (creature + token)");
    // Net hand: -1 cast Ally, +1 draw on the 2nd resolution = unchanged.
    assert_eq!(g.players[0].hand.len(), hand0, "drew only on the 2nd ETB");
}

/// Hermitic Herbalist's Lesson-only mana funds a Lesson spell but not a
/// non-Lesson spell of the same cost.
#[test]
fn hermitic_herbalist_lesson_only_mana() {
    use crate::mana::SpendRestriction;
    // Lesson-only blue mana pays for Boomerang Basics (a Lesson).
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let bb = g.add_card_to_hand(0, catalog::boomerang_basics());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_restricted(Color::Blue, 1, SpendRestriction::LessonSpellsOnly);
    g.perform_action(GameAction::CastSpell {
        card_id: bb, target: Some(Target::Permanent(mine)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lesson-only mana pays for a Lesson");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "bounced");

    // The same mana cannot pay for Unsummon (not a Lesson).
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let uns = g.add_card_to_hand(0, catalog::unsummon());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_restricted(Color::Blue, 1, SpendRestriction::LessonSpellsOnly);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: uns, target: Some(Target::Permanent(theirs)), additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "Lesson-only mana can't fund a non-Lesson spell");
}

/// Firebending Student's firebending X scales with its power: base 1/2 adds one
/// {R}; a +1/+1 counter (power 2) adds two.
#[test]
fn firebending_student_scales_with_power() {
    let mut g = two_player_game();
    let fs = g.add_card_to_battlefield(0, catalog::firebending_student());
    g.clear_sickness(fs);
    attack_with(&mut g, fs);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "power 1 → one {{R}}");

    let mut g = two_player_game();
    let fs = g.add_card_to_battlefield(0, catalog::firebending_student());
    g.battlefield_find_mut(fs).unwrap().add_counters(crate::card::CounterType::PlusOnePlusOne, 1);
    g.clear_sickness(fs);
    attack_with(&mut g, fs);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2, "power 2 → two {{R}}");
}

/// Boomerang Basics bounces a nonland permanent; it draws only when you
/// controlled the bounced permanent.
#[test]
fn boomerang_basics_draws_on_own_permanent() {
    // Bouncing your own creature draws.
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let bb = g.add_card_to_hand(0, catalog::boomerang_basics());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bb, target: Some(Target::Permanent(mine)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "own creature bounced");
    // -1 cast from hand, +1 bounced creature, +1 draw = net +1.
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew from controlling it");

    // Bouncing an opponent's creature does not draw.
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bb = g.add_card_to_hand(0, catalog::boomerang_basics());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bb, target: Some(Target::Permanent(theirs)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "opponent creature bounced");
    assert_eq!(g.players[0].hand.len(), hand - 1, "no draw; only spent the spell");
}

/// Fire Nation Cadets gains firebending 2 only while a Lesson sits in the gy.
#[test]
fn fire_nation_cadets_conditional_firebending() {
    let mut g = two_player_game();
    let fc = g.add_card_to_battlefield(0, catalog::fire_nation_cadets());
    assert!(!g.computed_permanent(fc).unwrap().keywords.contains(&Keyword::Firebending(2)),
        "no Lesson → no firebending");
    g.add_card_to_graveyard(0, catalog::yip_yip()); // a Lesson
    assert!(g.computed_permanent(fc).unwrap().keywords.contains(&Keyword::Firebending(2)),
        "Lesson in gy → firebending 2");
}

/// Firebending Lesson deals 2 normally and 5 when kicked.
#[test]
fn firebending_lesson_kicker_scales_damage() {
    for (kicked, dealt) in [(false, 2), (true, 5)] {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
        let fl = g.add_card_to_hand(0, catalog::firebending_lesson());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        if kicked { g.players[0].mana_pool.add_colorless(4); }
        let act = if kicked {
            GameAction::CastSpellKicked { card_id: fl, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None }
        } else {
            GameAction::CastSpell { card_id: fl, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None }
        };
        g.perform_action(act).expect("cast");
        drain_stack(&mut g);
        let t = g.computed_permanent(foe).unwrap().toughness;
        let dmg = g.battlefield_find(foe).unwrap().damage;
        assert_eq!(dmg, dealt, "kicked={kicked}: {dealt} marked (toughness {t})");
    }
}

/// Mongoose Lizard pings any target for 1 on ETB.
#[test]
fn mongoose_lizard_etb_pings() {
    let mut g = two_player_game();
    let ml = g.add_card_to_battlefield(0, catalog::mongoose_lizard());
    let before = g.players[1].life;
    g.fire_self_etb_triggers(ml, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 1, "ETB deals 1");
}

/// Origin of Metalbending mode 0 destroys an artifact.
#[test]
fn origin_of_metalbending_destroys_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    let om = g.add_card_to_hand(0, catalog::origin_of_metalbending());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: om, target: Some(Target::Permanent(art)), additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
}

/// Deadly Precision destroys a creature, paying the {4} when no creature/artifact
/// is available to sacrifice.
#[test]
fn deadly_precision_destroys_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dp = g.add_card_to_hand(0, catalog::deadly_precision());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4); // pays the {4} alternative
    g.perform_action(GameAction::CastSpell {
        card_id: dp, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "creature destroyed");
}

/// Enter the Avatar State grants the four keywords to a creature you control.
#[test]
fn enter_the_avatar_state_grants_keywords() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eas = g.add_card_to_hand(0, catalog::enter_the_avatar_state());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: eas, target: Some(Target::Permanent(mine)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let kws = g.computed_permanent(mine).unwrap().keywords;
    for kw in [Keyword::Flying, Keyword::FirstStrike, Keyword::Lifelink, Keyword::Hexproof] {
        assert!(kws.contains(&kw), "granted {kw:?}");
    }
}

/// Seismic Sense looks at (lands you control) cards and pulls a creature/land
/// to hand.
#[test]
fn seismic_sense_digs_for_a_land() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest()); // 2 lands → look at 2
    g.add_card_to_library(0, catalog::grizzly_bears()); // a creature on top
    g.add_card_to_library(0, catalog::lightning_bolt()); // a non-match below
    let ss = g.add_card_to_hand(0, catalog::seismic_sense());
    let hand0 = g.players[0].hand.len();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ss, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // Spell leaves hand (-1), the revealed creature is taken to hand (+1).
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "creature pulled to hand");
    assert_eq!(g.players[0].hand.len(), hand0, "net hand unchanged (spell out, card in)");
}

/// Earth Kingdom General earthbends a land you control on ETB.
#[test]
fn earth_kingdom_general_earthbends() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let ekg = g.add_card_to_battlefield(0, catalog::earth_kingdom_general());
    g.fire_self_etb_triggers(ekg, 0);
    drain_stack(&mut g);
    assert!(g.computed_permanent(land).unwrap().card_types.contains(&crate::card::CardType::Creature),
        "land became a creature");
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne), 2,
        "earthbend 2 counters");
}

/// Cruel Administrator mints a firebending Soldier when it attacks.
#[test]
fn cruel_administrator_attack_makes_soldier() {
    let mut g = two_player_game();
    let ca = g.add_card_to_battlefield(0, catalog::cruel_administrator());
    attack_with(&mut g, ca);
    assert_eq!(count_named(&g, 0, "Soldier"), 1, "made a Soldier token");
}

/// Sparring Dummy mills and pulls a land milled this way to hand.
#[test]
fn sparring_dummy_mills_for_land() {
    let mut g = two_player_game();
    let sd = g.add_card_to_battlefield(0, catalog::sparring_dummy());
    g.clear_sickness(sd);
    g.add_card_to_library(0, catalog::forest()); // top → milled, taken to hand
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sd, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "land to hand");
}

/// Buzzard-Wasp Colony's ETB sacrifices a creature to draw a card.
#[test]
fn buzzard_wasp_sacrifices_to_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bw = g.add_card_to_battlefield(0, catalog::buzzard_wasp_colony());
    let hand0 = g.players[0].hand.len();
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    g.fire_self_etb_triggers(bw, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card");
}

/// Jet's Brainwashing only steals the creature when kicked.
#[test]
fn jets_brainwashing_kicked_steals() {
    // Unkicked: just can't-block, no control change.
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let jb = g.add_card_to_hand(0, catalog::jets_brainwashing());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: jb, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(foe).unwrap().controller, 1, "still opponent's");

    // Kicked: steal it.
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let jb = g.add_card_to_hand(0, catalog::jets_brainwashing());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: jb, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(foe).unwrap().controller, 0, "stolen");
}

/// Meteor Sword destroys a permanent on ETB and buffs its wielder.
#[test]
fn meteor_sword_etb_destroys() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ms = g.add_card_to_battlefield(0, catalog::meteor_sword());
    g.fire_self_etb_triggers(ms, 0); // sole legal target → auto-targeted
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "permanent destroyed");
}

/// Kyoshi Battle Fan mints an Ally and attaches to it on ETB.
#[test]
fn kyoshi_battle_fan_living_weapon() {
    let mut g = two_player_game();
    let fan = g.add_card_to_battlefield(0, catalog::kyoshi_battle_fan());
    g.fire_self_etb_triggers(fan, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Ally"), 1, "minted an Ally");
    // The Ally carries the +1/+0 from the attached Fan.
    let ally = g.battlefield.iter().find(|c| c.definition.name == "Ally").unwrap().id;
    assert_eq!(g.computed_permanent(ally).unwrap().power, 2, "Ally is 2/1 with the Fan");
}

/// Bumi Bash mode 0 deals damage equal to the lands you control.
#[test]
fn bumi_bash_burns_for_lands() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::mountain()); }
    let foe = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
    let bb = g.add_card_to_hand(0, catalog::bumi_bash());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: bb, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 3, "3 lands → 3 damage");
}

/// Exhaust (CR 702.177): Rebellious Captives' Exhaust ability buffs and
/// earthbends once, then can't be used again.
#[test]
fn rebellious_captives_exhaust_once() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest()); // earthbend target
    let rc = g.add_card_to_battlefield(0, catalog::rebellious_captives());
    g.clear_sickness(rc);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rc, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("first activation");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rc).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne), 2,
        "two +1/+1 counters");
    // Second activation is illegal — Exhaust is once per game.
    g.players[0].mana_pool.add_colorless(6);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: rc, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).is_err(), "Exhaust can't fire twice");
}

/// Mai, Jaded Edge's Exhaust grants a double strike counter (CR 122 keyword
/// counter + 702.4).
#[test]
fn mai_exhaust_grants_double_strike() {
    let mut g = two_player_game();
    let mai = g.add_card_to_battlefield(0, catalog::mai_jaded_edge());
    g.clear_sickness(mai);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mai, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(mai).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "double strike counter grants the keyword");
}

/// Rough Rhino Cavalry's Exhaust pumps it and grants trample.
#[test]
fn rough_rhino_exhaust_pumps_and_tramples() {
    let mut g = two_player_game();
    let rr = g.add_card_to_battlefield(0, catalog::rough_rhino_cavalry());
    g.clear_sickness(rr);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(8);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rr, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(rr).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7), "5/5 + two counters");
    assert!(cp.keywords.contains(&Keyword::Trample), "gained trample");
}

/// Path to Redemption locks down a creature, then exiles it for an Ally.
#[test]
fn path_to_redemption_locks_then_exiles() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::path_to_redemption());
    ready0(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    assert!(g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::CantAttack),
        "enchanted creature can't attack");
    // Sacrifice the Aura to exile the creature and make an Ally.
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate sac ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "creature exiled");
    assert_eq!(count_named(&g, 0, "Ally"), 1, "made an Ally");
}

/// Dai Li Agents earthbends twice on ETB.
#[test]
fn dai_li_agents_double_earthbend() {
    let mut g = two_player_game();
    let l1 = g.add_card_to_battlefield(0, catalog::forest());
    let l2 = g.add_card_to_battlefield(0, catalog::forest());
    let dla = g.add_card_to_battlefield(0, catalog::dai_li_agents());
    g.fire_self_etb_triggers(dla, 0);
    drain_stack(&mut g);
    let counters: i32 = [l1, l2].iter()
        .map(|&l| g.battlefield_find(l).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne) as i32)
        .sum();
    assert_eq!(counters, 2, "two earthbend counters placed");
}

/// Fire Nation Warship investigates when it dies.
#[test]
fn fire_nation_warship_dies_to_clue() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::fire_nation_warship());
    let ctx = crate::game::effects::EffectContext::for_ability(ship, 0, Some(Target::Permanent(ship)));
    g.resolve_effect(&crate::effect::Effect::Destroy { what: crate::effect::Selector::Target(0) }, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"), "dies → Clue");
}

/// Earth Rumble Wrestlers pumps while you control a land creature.
#[test]
fn earth_rumble_wrestlers_conditional_pump() {
    let mut g = two_player_game();
    let erw = g.add_card_to_battlefield(0, catalog::earth_rumble_wrestlers());
    let base = g.computed_permanent(erw).unwrap();
    assert_eq!(base.power, 3, "base 3 power");
    assert!(!base.keywords.contains(&Keyword::Trample), "no trample at rest");
    // A land entering under your control this turn satisfies the landfall branch.
    g.players[0].lands_played_this_turn += 1;
    let cp = g.computed_permanent(erw).unwrap();
    assert_eq!(cp.power, 4, "+1/+0 after a land this turn");
    assert!(cp.keywords.contains(&Keyword::Trample), "and trample");
}

/// Abandon Attachments loots: discard one, draw two.
#[test]
fn abandon_attachments_loots() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::forest()); // discard fodder
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let aa = g.add_card_to_hand(0, catalog::abandon_attachments());
    ready0(&mut g);
    let hand0 = g.players[0].hand.len(); // aa + forest
    g.perform_action(GameAction::CastSpell {
        card_id: aa, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // -aa (cast), -forest (discard), +2 draw = net hand0 - 1.
    assert_eq!(g.players[0].hand.len(), hand0 - 1, "discarded one, drew two");
}

/// Sokka draws when he attacks alongside another creature, but not alone.
#[test]
fn sokka_draws_with_a_friend() {
    // Solo attack → no draw.
    let mut g = two_player_game();
    let sokka = g.add_card_to_battlefield(0, catalog::sokka_lateral_strategist());
    g.add_card_to_library(0, catalog::island());
    let hand0 = g.players[0].hand.len();
    attack_with(&mut g, sokka);
    assert_eq!(g.players[0].hand.len(), hand0, "no draw attacking alone");

    // Two attackers → draw.
    let mut g = two_player_game();
    let sokka = g.add_card_to_battlefield(0, catalog::sokka_lateral_strategist());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.clear_sickness(sokka);
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: sokka, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "draws attacking with a friend");
}

/// The Mechanist makes a Clue whenever you cast a noncreature spell.
#[test]
fn the_mechanist_magecraft_clue() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_mechanist_aerial_artisan());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    ready0(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a noncreature spell");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"), "noncreature cast → Clue");
}

/// Iroh, Tea Master makes a Food on ETB.
#[test]
fn iroh_tea_master_makes_food() {
    let mut g = two_player_game();
    let iroh = g.add_card_to_battlefield(0, catalog::iroh_tea_master());
    g.fire_self_etb_triggers(iroh, 0);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"), "ETB → Food");
}

/// Ty Lee, Chi Blocker taps a creature and stops its next untap.
#[test]
fn ty_lee_taps_and_locks() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ty = g.add_card_to_battlefield(0, catalog::ty_lee_chi_blocker());
    g.fire_self_etb_triggers(ty, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "target tapped");
    assert!(g.battlefield_find(foe).unwrap().skip_next_untap, "untap lock set");
    // It stays tapped through its controller's untap step.
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(foe).unwrap().tapped, "skips its next untap");
}

/// The Boulder earthbends X = your power-4+ creatures when it attacks.
#[test]
fn the_boulder_attack_earthbends_for_big_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest()); // earthbend target
    let boulder = g.add_card_to_battlefield(0, catalog::the_boulder_ready_to_rumble()); // 4/4
    attack_with(&mut g, boulder);
    // Only The Boulder (power 4) qualifies → X = 1 counter placed across lands.
    let counters: u32 = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_land())
        .map(|c| c.counter_count(crate::card::CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(counters, 1, "earthbend 1 (one power-4+ creature)");
}

/// The Earth King makes a 4/4 Bear on ETB.
#[test]
fn the_earth_king_makes_a_bear() {
    let mut g = two_player_game();
    let ek = g.add_card_to_battlefield(0, catalog::the_earth_king());
    g.fire_self_etb_triggers(ek, 0);
    drain_stack(&mut g);
    let bear = g.battlefield.iter().find(|c| c.definition.name == "Bear").map(|c| c.id);
    assert!(bear.is_some(), "made a Bear");
    let cp = g.computed_permanent(bear.unwrap()).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// The Lion-Turtle gains 3 life on ETB and taps for any color.
#[test]
fn the_lion_turtle_gains_and_ramps() {
    let mut g = two_player_game();
    let lt = g.add_card_to_battlefield(0, catalog::the_lion_turtle());
    let life0 = g.players[0].life;
    g.fire_self_etb_triggers(lt, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 3, "ETB gains 3 life");
    g.clear_sickness(lt);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Color(crate::mana::Color::Blue),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: lt, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    assert!(g.players[0].mana_pool.amount(crate::mana::Color::Blue) >= 1, "added a blue");
}

/// Suki anthems your other creatures and replaces a leaving permanent with an
/// Suki anthems your other creatures (+1/+0) but not herself.
#[test]
fn suki_anthems_other_creatures() {
    let mut g = two_player_game();
    let suki = g.add_card_to_battlefield(0, catalog::suki_courageous_rescuer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "other creature gets +1/+0");
    assert_eq!(g.computed_permanent(suki).unwrap().power, 2, "Suki doesn't pump herself");
}

/// Guru Pathik digs five deep and pulls a Lesson to hand.
#[test]
fn guru_pathik_digs_for_a_lesson() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::boomerang_basics()); // a Lesson on top
    for _ in 0..4 { g.add_card_to_library(0, catalog::mountain()); }
    let gp = g.add_card_to_battlefield(0, catalog::guru_pathik());
    g.fire_self_etb_triggers(gp, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Boomerang Basics"),
        "pulled a Lesson to hand");
}

/// Ty Lee, Artful Acrobat can pay {1} on attack to stop a blocker.
#[test]
fn ty_lee_artful_pays_to_unblock() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ty = g.add_card_to_battlefield(0, catalog::ty_lee_artful_acrobat());
    g.clear_sickness(ty);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.players[0].mana_pool.add_colorless(1); // available when the trigger resolves
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ty, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::CantBlock),
        "paid {{1}} → target can't block");
}

/// Uncle Iroh discounts your Lesson spells by {1}.
#[test]
fn uncle_iroh_discounts_lessons() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::uncle_iroh());
    let lesson = crate::card::CardInstance::new(g.next_id(), catalog::firebending_lesson(), 0);
    let nonlesson = crate::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &lesson, None), 1, "Lesson costs {{1}} less");
    assert_eq!(cost_reduction_for_spell(&g, 0, &nonlesson, None), 0, "non-Lesson unaffected");
}

/// Vindictive Warden pings each opponent for 1.
#[test]
fn vindictive_warden_pings_opponents() {
    let mut g = two_player_game();
    let vw = g.add_card_to_battlefield(0, catalog::vindictive_warden());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    let before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: vw, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 1, "1 damage to the opponent");
}

// ── Batch 11 ────────────────────────────────────────────────────────────────

/// Air Nomad Legacy makes a Clue and anthems your flyers.
#[test]
fn air_nomad_legacy_clue_and_flyer_anthem() {
    let mut g = two_player_game();
    let owl = g.add_card_to_battlefield(0, catalog::cat_owl()); // 3/3 flyer
    let leg = g.add_card_to_battlefield(0, catalog::air_nomad_legacy());
    g.fire_self_etb_triggers(leg, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Clue"), 1, "made a Clue");
    assert_eq!(g.computed_permanent(owl).unwrap().power, 4, "flyer gets +1/+1");
}

/// True Ancestry returns a permanent card from the graveyard and makes a Clue.
#[test]
fn true_ancestry_returns_and_clues() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ctx = crate::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&catalog::true_ancestry().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "card returned to hand");
    assert_eq!(count_named(&g, 0, "Clue"), 1, "made a Clue");
}

/// Tolls of War makes a Clue on ETB and an Ally when you sacrifice on your turn.
#[test]
fn tolls_of_war_etb_clue_and_sac_ally() {
    let mut g = two_player_game();
    let tolls = g.add_card_to_battlefield(0, catalog::tolls_of_war());
    g.fire_self_etb_triggers(tolls, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Clue"), 1, "ETB Clue");
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.sacrifice_one(fodder, 0, &mut vec![]);
    g.dispatch_triggers_for_events(&[GameEvent::PermanentSacrificed { card_id: fodder, who: 0 }]);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Ally"), 1, "sacrifice made an Ally");
}

/// Long Feng counters a creature you control when another one dies.
#[test]
fn long_feng_grows_on_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::long_feng_grand_secretariat());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // counter target
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut events = g.remove_to_graveyard_with_triggers(fodder);
    events.push(GameEvent::CreatureDied { card_id: fodder });
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let counters: u32 = g.battlefield.iter()
        .map(|c| c.counter_count(crate::card::CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(counters, 1, "one +1/+1 counter placed");
}

/// Zhao pumps your team when you sacrifice another permanent.
#[test]
fn zhao_ruthless_admiral_pumps_on_sac() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::zhao_ruthless_admiral());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.sacrifice_one(fodder, 0, &mut vec![]);
    g.dispatch_triggers_for_events(&[GameEvent::PermanentSacrificed { card_id: fodder, who: 0 }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "team +1/+0");
}

/// Zuko, Exiled Prince exiles the top card to play this turn.
#[test]
fn zuko_exiled_prince_impulses() {
    let mut g = two_player_game();
    let zuko = g.add_card_to_battlefield(0, catalog::zuko_exiled_prince());
    g.clear_sickness(zuko);
    g.add_card_to_library(0, catalog::lightning_bolt());
    let lib0 = g.players[0].library.len();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: zuko, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib0 - 1, "top card left the library");
    assert_eq!(g.exile.len(), 1, "exiled the top card");
}

/// Beifong's Bounty Hunters earthbends for a dying creature's power.
#[test]
fn beifongs_bounty_hunters_earthbends_for_power() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::beifongs_bounty_hunters());
    let fodder = g.add_card_to_battlefield(0, catalog::cat_gator()); // 3/2
    let mut events = g.remove_to_graveyard_with_triggers(fodder);
    events.push(GameEvent::CreatureDied { card_id: fodder });
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(land).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne),
        3,
        "earthbend 3 (Cat-Gator's power)"
    );
}

// ── Batch 12 ────────────────────────────────────────────────────────────────

/// Tundra Tank grants indestructible to a creature you control on ETB.
#[test]
fn tundra_tank_grants_indestructible() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tank = g.add_card_to_battlefield(0, catalog::tundra_tank());
    g.fire_self_etb_triggers(tank, 0);
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Indestructible));
}

/// Twin Blades attaches on ETB, granting +1/+1 and double strike.
#[test]
fn twin_blades_attaches_and_grants_double_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blades = g.add_card_to_battlefield(0, catalog::twin_blades());
    g.fire_self_etb_triggers(blades, 0);
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "equipped +1/+1");
    assert!(cp.keywords.contains(&Keyword::DoubleStrike), "gains double strike");
}

/// Vengeful Villagers taps an opponent's creature when it attacks.
#[test]
fn vengeful_villagers_taps_on_attack() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let vv = g.add_card_to_battlefield(0, catalog::vengeful_villagers());
    attack_with(&mut g, vv);
    assert!(g.battlefield_find(foe).unwrap().tapped, "opponent creature tapped");
}

/// Invasion Tactics pumps your team +2/+2 on ETB.
#[test]
fn invasion_tactics_pumps_team() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let it = g.add_card_to_battlefield(0, catalog::invasion_tactics());
    g.fire_self_etb_triggers(it, 0);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "team +2/+2");
}

/// Jet, Freedom Fighter pings for the number of creatures you control on ETB.
#[test]
fn jet_freedom_fighter_etb_pings() {
    let mut g = two_player_game();
    let jet = g.add_card_to_battlefield(0, catalog::jet_freedom_fighter());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // count = 2
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.fire_self_etb_triggers(jet, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "2 damage killed the 2/2");
}

/// Sold Out exiles a creature and clues only if it was dealt damage.
#[test]
fn sold_out_clues_on_damaged() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::cat_gator()); // 3/2
    let mut events = vec![];
    g.deal_damage_to_from(crate::game::effects::EntityRef::Permanent(foe), 1, None, &mut events);
    let ctx = crate::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(foe)), 0, 0);
    g.resolve_effect(&catalog::sold_out().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == foe), "creature exiled");
    assert_eq!(count_named(&g, 0, "Clue"), 1, "made a Clue (was damaged)");
}

/// Sokka, Tenacious Tactician gives other Allies menace.
#[test]
fn sokka_tenacious_lords_allies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sokka_tenacious_tactician());
    let ally = g.add_card_to_battlefield(0, catalog::compassionate_healer()); // Ally
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Menace),
        "other Ally gains menace");
}

/// Team Avatar pumps a lone attacker by the number of creatures you control.
#[test]
fn team_avatar_pumps_lone_attacker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::team_avatar());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack_with(&mut g, bear);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+1 (one creature)");
}

// ── Batch 13 ────────────────────────────────────────────────────────────────

/// Appa, Loyal Sky Bison's ETB mode grants flying to a creature you control.
#[test]
fn appa_loyal_grants_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let appa = g.add_card_to_battlefield(0, catalog::appa_loyal_sky_bison());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Modes(vec![0]),
        crate::decision::DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.fire_self_etb_triggers(appa, 0);
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying));
}

/// Fire Lord Azula copies a spell cast while she's attacking.
#[test]
fn fire_lord_azula_copies_while_attacking() {
    let mut g = two_player_game();
    let azula = g.add_card_to_battlefield(0, catalog::fire_lord_azula());
    g.battlefield_find_mut(azula).unwrap().attacked_this_turn = true;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    let life0 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life0 - 6, "original + copy each deal 3");
}

/// Rockalanche earthbends for the number of Forests you control.
#[test]
fn rockalanche_earthbends_for_forests() {
    let mut g = two_player_game();
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    let rock = g.add_card_to_hand(0, catalog::rockalanche());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: rock, target: Some(Target::Permanent(f1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Rockalanche");
    drain_stack(&mut g);
    let counters: u32 = g.battlefield.iter()
        .filter(|c| c.definition.name == "Forest")
        .map(|c| c.counter_count(crate::card::CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(counters, 2, "earthbend 2 (two Forests)");
}

/// Fire Nation Attacks makes two firebending Soldiers.
#[test]
fn fire_nation_attacks_makes_two_soldiers() {
    let mut g = two_player_game();
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::fire_nation_attacks().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Soldier"), 2, "two Soldier tokens");
}

// ── Batch 14 ────────────────────────────────────────────────────────────────

/// How to Start a Riot grants menace and pumps the target player's creatures.
#[test]
fn how_to_start_a_riot_menace_and_pump() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let riot = g.add_card_to_hand(0, catalog::how_to_start_a_riot());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: riot,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    }).expect("cast riot");
    drain_stack(&mut g);
    assert!(g.computed_permanent(mine).unwrap().keywords.contains(&Keyword::Menace), "menace");
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 4, "target player's creatures +2/+0");
}

/// Lost Days tucks a creature into its owner's library and makes a Clue.
#[test]
fn lost_days_tucks_and_clues() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = crate::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(foe)), 0, 0);
    g.resolve_effect(&catalog::lost_days().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "creature left the battlefield");
    assert!(g.players[1].library.iter().any(|c| c.id == foe), "tucked into owner's library");
    assert_eq!(count_named(&g, 0, "Clue"), 1, "made a Clue");
}

/// Sokka's Haiku counters a spell and untaps a land.
#[test]
fn sokkas_haiku_counters_and_untaps() {
    let mut g = two_player_game();
    // Opponent casts a creature spell.
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts");
    // Seat 0 responds with Sokka's Haiku, untapping a tapped land.
    let land = g.add_card_to_battlefield(0, catalog::island());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    let haiku = g.add_card_to_hand(0, catalog::sokkas_haiku());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: haiku,
        target: Some(Target::Permanent(spell)),
        additional_targets: vec![Target::Permanent(land)],
        mode: None,
        x_value: None,
    }).expect("cast haiku");
    drain_stack(&mut g);
    assert!(g.battlefield_find(spell).is_none(), "the creature spell was countered");
    assert!(!g.battlefield_find(land).unwrap().tapped, "land untapped");
}

// ── Batch 15 ────────────────────────────────────────────────────────────────

/// Crescent Island Temple mints one Monk per Shrine you control on ETB.
#[test]
fn crescent_island_temple_mints_per_shrine() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::southern_air_temple()); // a second Shrine
    let crescent = g.add_card_to_battlefield(0, catalog::crescent_island_temple());
    g.fire_self_etb_triggers(crescent, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Monk"), 2, "one Monk per Shrine (2 Shrines)");
}

/// Southern Air Temple counters each creature by the number of Shrines.
#[test]
fn southern_air_temple_counters_per_shrine() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::crescent_island_temple()); // a second Shrine
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let temple = g.add_card_to_battlefield(0, catalog::southern_air_temple());
    g.fire_self_etb_triggers(temple, 0);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne),
        2,
        "+1/+1 per Shrine (2 Shrines)"
    );
}

/// Waterbending Scroll's draw ability is discounted by your Islands.
#[test]
fn waterbending_scroll_island_discount() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let scroll = g.add_card_to_battlefield(0, catalog::waterbending_scroll());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(4); // {6} - 2 Islands = {4}
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: scroll, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate at the discounted cost");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew after the Island discount");
}

// ── Batch 16 ────────────────────────────────────────────────────────────────

/// Kyoshi Island Plaza fetches one basic per Shrine onto the battlefield tapped.
#[test]
fn kyoshi_island_plaza_fetches_per_shrine() {
    let mut g = two_player_game();
    let fetch = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(fetch)),
    ]));
    let plaza = g.add_card_to_battlefield(0, catalog::kyoshi_island_plaza());
    g.fire_self_etb_triggers(plaza, 0);
    drain_stack(&mut g);
    let bf = g.battlefield.iter().find(|c| c.id == fetch);
    assert!(bf.is_some(), "one basic for one Shrine");
    assert!(bf.unwrap().tapped, "enters tapped");
}

/// Wan Shi Tong enters with X +1/+1 counters and draws half X.
#[test]
fn wan_shi_tong_enters_with_x_counters() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let wan = g.add_card_to_hand(0, catalog::wan_shi_tong_librarian());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(4); // X = 4
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    let lib0 = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: wan, target: None, additional_targets: vec![], mode: None, x_value: Some(4),
    }).expect("cast for X=4");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(wan).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne),
        4,
        "X +1/+1 counters"
    );
    assert_eq!(g.players[0].library.len(), lib0 - 2, "drew half of 4 = 2");
}

// ── Batch 17 ────────────────────────────────────────────────────────────────

/// Bender's Waterskin taps for one mana of any color.
#[test]
fn benders_waterskin_taps_for_any_color() {
    let mut g = two_player_game();
    let skin = g.add_card_to_battlefield(0, catalog::benders_waterskin());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Color(crate::mana::Color::Blue),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: skin, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    assert!(g.players[0].mana_pool.amount(crate::mana::Color::Blue) >= 1, "added blue");
}

/// The Fire Nation Drill's ETB may tap to destroy a small creature.
#[test]
fn fire_nation_drill_etb_destroys_small_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, power ≤ 4
    let drill = g.add_card_to_battlefield(0, catalog::the_fire_nation_drill());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
        crate::decision::DecisionAnswer::Target(Target::Permanent(foe)),
    ]));
    g.fire_self_etb_triggers(drill, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "small creature destroyed");
    assert!(g.battlefield_find(drill).unwrap().tapped, "Drill tapped itself");
}

/// Iroh, Grand Lotus grants flashback to instants/sorceries in your graveyard.
#[test]
fn iroh_grand_lotus_flashbacks_graveyard_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::iroh_grand_lotus());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFlashback {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Iroh grants flashback");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "flashed-back Bolt dealt 3");
    assert!(g.exile.iter().any(|c| c.id == bolt), "flashback exiles the spell");
}
/// Razor Rings deals 4 to an attacking creature and gains life = excess damage.
#[test]
fn razor_rings_burns_attacker_for_excess_life() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    attack_with(&mut g, bears);
    let rings = g.add_card_to_hand(0, catalog::razor_rings());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: rings, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Razor Rings");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "4 damage kills the 2/2");
    assert_eq!(g.players[0].life, life + 2, "gain life = excess (4 - 2)");
}

/// The Last Agni Kai fights and adds {R} equal to the excess damage dealt.
#[test]
fn last_agni_kai_fights_and_adds_red_for_excess() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let kai = g.add_card_to_hand(0, catalog::the_last_agni_kai());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: kai, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast The Last Agni Kai");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "4-power kills the 2/2");
    assert_eq!(g.players[0].mana_pool.amount(crate::mana::Color::Red), 2, "excess 2 → {{R}}{{R}}");
}

/// Hei Bai sacrifices for two +1/+1 counters, then moves them on leaving.
#[test]
fn hei_bai_sacrifices_then_moves_counters_on_leave() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    let hei_bai = g.add_card_to_battlefield(0, catalog::hei_bai_spirit_of_balance());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.fire_self_etb_triggers(hei_bai, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.computed_permanent(hei_bai).unwrap().power, 5, "3/3 + two counters = 5/5");
    // LTB moves the two counters to the only other creature.
    let keeper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_to_graveyard_with_triggers(hei_bai);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(keeper).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "Hei Bai's counters moved to the survivor");
}

/// Sun Warriors' firebending X = the number of creatures you control.
#[test]
fn sun_warriors_firebends_per_creature() {
    let mut g = two_player_game();
    let sw = g.add_card_to_battlefield(0, catalog::sun_warriors());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(sw);
    attack_with(&mut g, sw);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3, "3 creatures → three {{R}}");
}

/// Suki's power tracks creatures you control; attacking mints a tapped Ally.
#[test]
fn suki_power_tracks_creatures_and_mints_ally() {
    let mut g = two_player_game();
    let suki = g.add_card_to_battlefield(0, catalog::suki_kyoshi_warrior());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(suki).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4), "2 creatures → 2/4");
    g.clear_sickness(suki);
    attack_with(&mut g, suki);
    assert_eq!(count_named(&g, 0, "Ally"), 1, "attack minted an Ally token");
}

/// Toph earthbends on enter; her power equals counters on lands you control.
#[test]
fn toph_blind_bandit_power_tracks_land_counters() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    let toph = g.add_card_to_battlefield(0, catalog::toph_the_blind_bandit());
    g.fire_self_etb_triggers(toph, 0);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(toph).unwrap().power, 2, "earthbend 2 → power 2");
}

/// Cycle of Renewal sacrifices a land and fetches two basics tapped.
#[test]
fn cycle_of_renewal_sacs_then_fetches_two() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let f1 = g.add_card_to_library(0, catalog::forest());
    let f2 = g.add_card_to_library(0, catalog::forest());
    let cr = g.add_card_to_hand(0, catalog::cycle_of_renewal());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(f1)),
        crate::decision::DecisionAnswer::Search(Some(f2)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: cr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cycle of Renewal");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "a land was sacrificed");
    let lands = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    assert_eq!(lands, 2, "fetched two basics (net: -1 sacced +2 fetched)");
}

/// Zuko's Exile exiles a permanent and gives its controller a Clue.
#[test]
fn zukos_exile_exiles_and_gifts_clue() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ze = g.add_card_to_hand(0, catalog::zukos_exile());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: ze, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Zuko's Exile");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "creature exiled");
    assert_eq!(count_named(&g, 1, "Clue"), 1, "its controller got a Clue");
}

/// Sokka gains a +1/+1 counter when you cast a Lesson spell.
#[test]
fn sokka_grows_on_lesson_cast() {
    let mut g = two_player_game();
    let sokka = g.add_card_to_battlefield(0, catalog::sokka_bold_boomeranger());
    let lesson = g.add_card_to_hand(0, catalog::combustion_technique()); // {1}{R} Lesson
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpell {
        card_id: lesson, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Lesson");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sokka).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne), 1,
        "Sokka grew from the Lesson cast");
}

/// Zuko's Conviction returns a gy creature to hand, or reanimates it if kicked.
#[test]
fn zukos_conviction_kicked_reanimates() {
    let mut g = two_player_game();
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let zc = g.add_card_to_hand(0, catalog::zukos_conviction());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: zc, target: Some(Target::Permanent(bears)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    let b = g.battlefield_find(bears).expect("reanimated onto battlefield");
    assert!(b.tapped, "enters tapped");
}

/// Barrels of Blasting Jelly sacrifices to deal 5 to a creature.
#[test]
fn barrels_of_blasting_jelly_burns() {
    let mut g = two_player_game();
    let barrels = g.add_card_to_battlefield(0, catalog::barrels_of_blasting_jelly());
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: barrels, ability_index: 1, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], x_value: None,
    }).expect("activate sac-burn");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "5 damage kills the 4/4");
    assert!(g.battlefield_find(barrels).is_none(), "Barrels sacrificed");
}

/// Accumulate Wisdom digs three and takes all three with 3+ Lessons in gy.
#[test]
fn accumulate_wisdom_takes_all_with_lessons() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::combustion_technique()); } // Lessons
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let aw = g.add_card_to_hand(0, catalog::accumulate_wisdom());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: aw, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3, "took all three (cast 1, drew 3)");
}

/// Dragonfly Swarm's power tracks noncreature, nonland cards in your graveyard.
#[test]
fn dragonfly_swarm_power_tracks_graveyard() {
    let mut g = two_player_game();
    let swarm = g.add_card_to_battlefield(0, catalog::dragonfly_swarm());
    assert_eq!(g.computed_permanent(swarm).unwrap().power, 0, "empty gy → 0 power");
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::combustion_technique());
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature: not counted
    assert_eq!(g.computed_permanent(swarm).unwrap().power, 2, "two noncreature/nonland cards");
}

/// Abandoned Air Temple enters tapped without a basic, untapped with one.
#[test]
fn abandoned_air_temple_conditional_tap() {
    let mut g = two_player_game();
    let t1 = g.move_card_to_battlefield_for_test(0, catalog::abandoned_air_temple());
    drain_stack(&mut g);
    assert!(g.battlefield_find(t1).unwrap().tapped, "no basic → enters tapped");

    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest()); // a basic land
    let t2 = g.move_card_to_battlefield_for_test(0, catalog::abandoned_air_temple());
    drain_stack(&mut g);
    assert!(!g.battlefield_find(t2).unwrap().tapped, "basic present → enters untapped");
}

/// Fire Nation Palace grants firebending 4 to a creature until end of turn.
#[test]
fn fire_nation_palace_grants_firebending() {
    let mut g = two_player_game();
    let pal = g.add_card_to_battlefield(0, catalog::fire_nation_palace());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pal, ability_index: 1, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("grant firebending");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Firebending(4)),
        "creature gained firebending 4");
}

/// Ba Sing Se's activated ability earthbends a target land.
#[test]
fn ba_sing_se_earthbends_a_land() {
    let mut g = two_player_game();
    let bss = g.add_card_to_battlefield(0, catalog::ba_sing_se());
    let target_land = g.add_card_to_battlefield(0, catalog::forest());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bss, ability_index: 1, target: Some(Target::Permanent(target_land)),
        additional_targets: vec![], x_value: None,
    }).expect("earthbend ability");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target_land).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne), 2,
        "earthbend 2 placed two counters on the land");
}

/// Price of Freedom destroys an opponent's artifact/land and draws a card.
#[test]
fn price_of_freedom_destroys_and_draws() {
    let mut g = two_player_game();
    let foe_land = g.add_card_to_battlefield(1, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let pf = g.add_card_to_hand(0, catalog::price_of_freedom());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: pf, target: Some(Target::Permanent(foe_land)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Price of Freedom");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe_land).is_none(), "opponent's land destroyed");
    assert_eq!(g.players[0].hand.len(), hand, "cast 1, drew 1 (net unchanged)");
}

/// Realm of Koh mints a Spirit token that only Spirits can block.
#[test]
fn realm_of_koh_makes_spirit() {
    let mut g = two_player_game();
    let realm = g.add_card_to_battlefield(0, catalog::realm_of_koh());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: realm, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("make Spirit");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Spirit"), 1, "minted a Spirit");
}


/// Earthen Ally's power tracks the distinct colors among Allies you control.
#[test]
fn earthen_ally_power_tracks_ally_colors() {
    let mut g = two_player_game();
    let ea = g.add_card_to_battlefield(0, catalog::earthen_ally()); // green Ally
    assert_eq!(g.computed_permanent(ea).unwrap().power, 1, "self is a green Ally → 1 color");
    g.add_card_to_battlefield(0, catalog::sun_warriors()); // red/white Ally → +R +W
    assert_eq!(g.computed_permanent(ea).unwrap().power, 3, "green + red + white = 3 colors");
}

/// Agna Qel'a loots (draw then discard) via its utility ability.
#[test]
fn agna_qela_loots() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::agna_qela());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to discard
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("loot");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "drew 1, discarded 1 (net unchanged)");
}

/// Leaves from the Vine: chapter I mills three and makes a Food; chapter III
/// draws because a creature/Lesson is in the graveyard.
#[test]
fn leaves_from_the_vine_saga_chapters() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); } // milled creatures
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // chapter II target
    let id = g.add_card_to_hand(0, catalog::leaves_from_the_vine());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Food"), 1, "chapter I made a Food");
    assert!(g.players[0].graveyard.iter().filter(|c| c.definition.is_creature()).count() >= 1,
        "chapter I milled creatures");
    g.saga_advance(id); // chapter II — +1/+1 on up to two creatures
    drain_stack(&mut g);
    let hand = g.players[0].hand.len();
    g.add_card_to_library(0, catalog::forest()); // a card to draw
    g.saga_advance(id); // chapter III — draw (creature in gy)
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "chapter III drew");
}

/// Rumble Arena scries on enter and taps for colorless or filtered any-color.
#[test]
fn rumble_arena_scries_and_taps() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let ra = g.move_card_to_battlefield_for_test(0, catalog::rumble_arena());
    drain_stack(&mut g); // ETB scry resolves (auto-decider keeps top)
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ra, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for C");
    assert!(g.players[0].mana_pool.colorless_amount() >= 1, "added colorless");
}

/// Hakoda's sacrifice gives your creatures +0/+5 and indestructible.
#[test]
fn hakoda_sacrifice_buffs_team() {
    let mut g = two_player_game();
    let hakoda = g.add_card_to_battlefield(0, catalog::hakoda_selfless_commander());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: hakoda, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sacrifice Hakoda");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.toughness, 7, "grizzly 2/2 +0/+5 = 2/7");
    assert!(cp.keywords.contains(&Keyword::Indestructible), "gained indestructible");
    assert!(g.battlefield_find(hakoda).is_none(), "Hakoda was sacrificed");
}

/// Momo grows when another flyer you control enters.
#[test]
fn momo_grows_on_flyer_entering() {
    let mut g = two_player_game();
    let momo = g.add_card_to_battlefield(0, catalog::momo_friendly_flier());
    assert_eq!(g.computed_permanent(momo).unwrap().power, 1, "base 1/1");
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Cast a flyer so Momo's "another flyer enters" watcher fires.
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Serra Angel");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(momo).unwrap().power, 2, "Momo grew +1/+1");
    // A non-flyer entering doesn't pump Momo.
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bears");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(momo).unwrap().power, 2, "ground creature doesn't trigger");
}

/// Obsessive Pursuit drains 1 and makes a Clue on ETB.
#[test]
fn obsessive_pursuit_etb_drain_clue() {
    let mut g = two_player_game();
    let op = g.add_card_to_battlefield(0, catalog::obsessive_pursuit());
    let life = g.players[0].life;
    g.fire_self_etb_triggers(op, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "lost 1 life");
    assert_eq!(count_named(&g, 0, "Clue"), 1, "made a Clue");
}

/// Obsessive Pursuit puts X counters (X = permanents sacrificed) on an
/// attacker, with lifelink at X≥3.
#[test]
fn obsessive_pursuit_attack_counters_and_lifelink() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::obsessive_pursuit());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].permanents_sacrificed_this_turn = 3;
    attack_with(&mut g, bear);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne), 3,
        "3 counters = 3 permanents sacrificed");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Lifelink),
        "X≥3 grants lifelink");
}

/// Combustion Man destroys the target unless its controller pays life = his
/// power; the AutoDecider declines, so the permanent dies.
#[test]
fn combustion_man_destroys_when_unpaid() {
    let mut g = two_player_game();
    let cm = g.add_card_to_battlefield(0, catalog::combustion_man());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    attack_with(&mut g, cm);
    assert!(g.battlefield_find(victim).is_none(), "target destroyed (life unpaid)");
}

/// Teo loots when a flyer attacks and adds a counter when a nonland is pitched.
#[test]
fn teo_loots_and_counters_on_nonland_discard() {
    let mut g = two_player_game();
    let teo = g.add_card_to_battlefield(0, catalog::teo_spirited_glider());
    g.players[0].hand.clear();
    g.add_card_to_library(0, catalog::grizzly_bears()); // nonland to draw then pitch
    attack_with(&mut g, teo); // Teo has flying
    assert_eq!(g.battlefield_find(teo).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne), 1,
        "nonland discard adds a counter");
}

/// Bitter Work draws when you attack with a power-4+ creature.
#[test]
fn bitter_work_draws_on_big_attacker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bitter_work());
    let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 flyer
    g.players[0].hand.clear();
    g.add_card_to_library(0, catalog::forest());
    attack_with(&mut g, big);
    assert_eq!(g.players[0].hand.len(), 1, "drew a card off the power-4 attacker");
}

/// Bitter Work's Exhaust earthbends 4 onto a land, once per game.
#[test]
fn bitter_work_exhaust_earthbends() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let bw = g.add_card_to_battlefield(0, catalog::bitter_work());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bw, ability_index: 0,
        target: Some(crate::game::types::Target::Permanent(land)),
        additional_targets: vec![], x_value: None,
    }).expect("earthbend");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne), 4,
        "earthbend 4 counters");
}

/// Sandbender Scavengers grows when you sacrifice another permanent.
#[test]
fn sandbender_grows_on_sacrifice() {
    use crate::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    let sb = g.add_card_to_battlefield(0, catalog::sandbender_scavengers());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crate::game::effects::EffectContext::for_ability(sb, 0, None);
    let evs = g.resolve_effect(
        &Effect::Sacrifice {
            who: Selector::You,
            count: Value::Const(1),
            filter: crate::card::SelectionRequirement::Creature
                .and(crate::card::SelectionRequirement::OtherThanSource),
        },
        &ctx,
    ).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sb).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne), 1,
        "grew from the sacrifice");
}

/// Sandbender Scavengers reanimates a creature with MV ≤ its last-known power.
#[test]
fn sandbender_reanimates_on_death() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let sb = g.add_card_to_battlefield(0, catalog::sandbender_scavengers());
    g.battlefield_find_mut(sb).unwrap().add_counters(crate::card::CounterType::PlusOnePlusOne, 2); // power 3
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2 ≤ 3
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),               // yes, exile + reanimate
        DecisionAnswer::Target(crate::game::types::Target::Permanent(bears)),
    ]));
    g.remove_to_graveyard_with_triggers(sb);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some(), "MV-2 creature reanimated");
    assert!(g.exile.iter().any(|c| c.id == sb), "Sandbender exiled itself");
}

/// Diligent Zookeeper buffs non-Humans by their creature-type count; Humans get
/// nothing.
#[test]
fn diligent_zookeeper_per_type_anthem() {
    let mut g = two_player_game();
    let zoo = g.add_card_to_battlefield(0, catalog::diligent_zookeeper());
    // Momo is Lemur Bat Ally (3 types, non-Human): +3/+3 → 4/4.
    let momo = g.add_card_to_battlefield(0, catalog::momo_friendly_flier());
    assert_eq!(g.computed_permanent(momo).unwrap().power, 4, "1/1 +3/+3 = 4 power");
    assert_eq!(g.computed_permanent(momo).unwrap().toughness, 4, "1/1 +3/+3 = 4 toughness");
    // The Zookeeper itself is Human — unaffected.
    assert_eq!(g.computed_permanent(zoo).unwrap().power, 4, "Human gets no bonus");
}

/// Katara, the Fearless makes an Ally's triggered ability fire an extra time.
#[test]
fn katara_doubles_ally_etb_trigger() {
    // Baseline: Kyoshi Warriors' ETB makes one Ally token.
    let mut g = two_player_game();
    let kw = g.add_card_to_battlefield(0, catalog::kyoshi_warriors());
    g.fire_self_etb_triggers(kw, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Ally"), 1, "no Katara → one token");

    // With Katara out, the Ally ETB triggers twice → two tokens.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::katara_the_fearless());
    let kw = g.add_card_to_battlefield(0, catalog::kyoshi_warriors());
    g.fire_self_etb_triggers(kw, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Ally"), 2, "Katara → ETB fires twice");
}

/// Katara only boosts Ally triggers, not a non-Ally creature's trigger.
#[test]
fn katara_ignores_non_ally_triggers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::katara_the_fearless());
    // Cat-Gator is a non-Ally (Crocodile) whose ETB pings for Swamps.
    g.add_card_to_battlefield(0, catalog::swamp());
    let cg = g.add_card_to_battlefield(0, catalog::cat_gator());
    let before = g.players[1].life;
    g.fire_self_etb_triggers(cg, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 1, "non-Ally ETB fires once (1 Swamp)");
}

/// Fire Lord Zuko counters the team when a permanent enters from exile.
#[test]
fn fire_lord_zuko_counters_on_enter_from_exile() {
    use crate::effect::{PlayerRef, ZoneDest};
    let mut g = two_player_game();
    let zuko = g.add_card_to_battlefield(0, catalog::fire_lord_zuko());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let exiled = g.add_card_to_exile(0, catalog::grizzly_bears());
    // Return the exiled creature to the battlefield (enters from exile).
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let mut events = Vec::new();
    g.move_card_to(
        exiled,
        &ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        &ctx,
        &mut events,
    );
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let pp = crate::card::CounterType::PlusOnePlusOne;
    assert_eq!(g.battlefield_find(zuko).unwrap().counter_count(pp), 1, "Zuko gets a counter");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(pp), 1, "bear gets a counter");
}

/// Raven Eagle exiles a graveyard creature on ETB and investigates for it.
#[test]
fn raven_eagle_etb_exiles_and_clues() {
    let mut g = two_player_game();
    let victim = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let re = g.add_card_to_battlefield(0, catalog::raven_eagle());
    g.fire_self_etb_triggers(re, 0);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "graveyard creature exiled");
    assert_eq!(count_named(&g, 0, "Clue"), 1, "creature exile → a Clue");
}

/// Raven Eagle drains 1 when you draw your second card in a turn.
#[test]
fn raven_eagle_second_draw_drains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::raven_eagle());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let div = g.add_card_to_hand(0, catalog::divination()); // draw 2 → second-draw fires
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp = g.players[1].life;
    let you = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: div, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Divination");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, you + 1, "you gained 1");
}

/// Fatal Fissure earthbends a land you control when the chosen creature dies.
#[test]
fn fatal_fissure_earthbends_on_target_death() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ff = g.add_card_to_hand(0, catalog::fatal_fissure());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ff, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Fatal Fissure");
    drain_stack(&mut g);
    // Kill the watched creature via SBA → delayed earthbend fires on our land.
    g.battlefield_find_mut(victim).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let pp = crate::card::CounterType::PlusOnePlusOne;
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(pp), 4, "earthbend 4 on the land");
    assert!(g.computed_permanent(land).unwrap().card_types.contains(&crate::card::CardType::Creature),
        "land became a creature");
}

/// Serpent of the Pass costs {1} less per noncreature, nonland gy card.
#[test]
fn serpent_of_the_pass_cost_reduction() {
    let mut g = two_player_game();
    // Three noncreature, nonland cards in graveyard → {3} off {5}{U}{U}.
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::lightning_bolt()); }
    let serp = g.add_card_to_hand(0, catalog::serpent_of_the_pass());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2); // {2}{U}{U} = reduced cost
    g.perform_action(GameAction::CastSpell {
        card_id: serp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast at reduced cost");
    drain_stack(&mut g);
    assert!(g.battlefield_find(serp).is_some(), "Serpent resolved with the discount");
}

/// Serpent of the Pass gains flash with 3+ Lessons in the graveyard.
#[test]
fn serpent_of_the_pass_conditional_flash() {
    let mut g = two_player_game();
    g.active_player_idx = 1; // opponent's turn → no sorcery-speed window for P0
    g.priority.player_with_priority = 0;
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::combustion_technique()); } // Lessons
    let serp = g.add_card_to_hand(0, catalog::serpent_of_the_pass());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 5);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: serp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("flash-cast off 3 Lessons");
    drain_stack(&mut g);
    assert!(g.battlefield_find(serp).is_some(), "flash let it resolve off-turn");
}

/// Without 3 Lessons, Serpent of the Pass can't be cast at instant speed.
#[test]
fn serpent_of_the_pass_no_flash_without_lessons() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    let serp = g.add_card_to_hand(0, catalog::serpent_of_the_pass());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 5);
    g.players[0].mana_pool.add_colorless(5);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: serp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "sorcery-speed only without the Lesson gate");
}

/// Earth Rumble earthbends a land and makes your creature fight an opponent's.
#[test]
fn earth_rumble_earthbends_and_fights() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let er = g.add_card_to_hand(0, catalog::earth_rumble());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: er, target: Some(Target::Permanent(land)),
        additional_targets: vec![Target::Permanent(mine), Target::Permanent(foe)],
        mode: None, x_value: None,
    }).expect("cast Earth Rumble");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne), 2,
        "earthbend 2 on the land");
    assert!(g.battlefield_find(foe).is_none(), "the 2/2 died to the 4/4 fight");
}

/// Allies at Last: Affinity for Allies discounts it, and two creatures each
/// deal their power to an opponent's creature.
#[test]
fn allies_at_last_affinity_and_double_strike_damage() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    // Two Allies you control discount the {2}{G} spell by {2} → {G}.
    let a1 = g.add_card_to_battlefield(0, catalog::kyoshi_warriors()); // 3/3 Ally
    let a2 = g.add_card_to_battlefield(0, catalog::momo_friendly_flier()); // 1/1 Ally
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let spell = g.add_card_to_hand(0, catalog::allies_at_last());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1); // only {G} — affinity must cover the rest
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a1)),
        additional_targets: vec![Target::Permanent(a2), Target::Permanent(foe)],
        mode: None, x_value: None,
    }).expect("cast at affinity-reduced cost");
    drain_stack(&mut g);
    // 3 + 1 = 4 damage to the 4/4 → it dies.
    assert!(g.battlefield_find(foe).is_none(), "4 total damage kills the 4/4");
}

/// Honest Work shrinks an opponent's creature to a 1/1 Citizen, taps it, and
/// strips its counters on ETB.
#[test]
fn honest_work_shrinks_and_taps() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer
    g.battlefield_find_mut(foe).unwrap().add_counters(crate::card::CounterType::PlusOnePlusOne, 2);
    let aura = g.add_card_to_hand(0, catalog::honest_work());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Honest Work");
    drain_stack(&mut g);
    let cp = g.computed_permanent(foe).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "becomes 1/1");
    assert!(!cp.keywords.contains(&Keyword::Flying), "loses flying (abilities stripped)");
    assert!(g.battlefield_find(foe).unwrap().tapped, "tapped on ETB");
    assert_eq!(g.battlefield_find(foe).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne), 0,
        "counters removed");
}

/// Bumi runs up to X modes (X = Lessons in graveyard). With 2 Lessons and the
/// default pick order, modes 0 (3 counters) and 1 (scry) run; mode 2 doesn't.
#[test]
fn bumi_choose_up_to_lessons() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest()); // an earthbend target if mode 2 fires
    // Two Lessons in graveyard → choose up to 2.
    g.add_card_to_graveyard(0, catalog::combustion_technique());
    g.add_card_to_graveyard(0, catalog::combustion_technique());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); } // scry needs a library
    let bumi = g.add_card_to_battlefield(0, catalog::bumi_king_of_three_trials());
    g.fire_self_etb_triggers(bumi, 0);
    drain_stack(&mut g);
    // Mode 0 ran → 3 counters on Bumi.
    assert_eq!(g.battlefield_find(bumi).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne), 3,
        "first mode put 3 counters on Bumi");
}

/// With zero Lessons, Bumi's ETB chooses nothing.
#[test]
fn bumi_no_lessons_chooses_nothing() {
    let mut g = two_player_game();
    let bumi = g.add_card_to_battlefield(0, catalog::bumi_king_of_three_trials());
    g.fire_self_etb_triggers(bumi, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bumi).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne), 0,
        "no Lessons → no modes run");
}
