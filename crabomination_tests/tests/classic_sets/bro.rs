//! Functionality tests for The Brothers' War (BRO) — the Prototype mechanic
//! (CR 702.160) and the prototype artifact creatures.

use crabomination::card::{CounterType, Keyword, WardCost};
use crabomination::catalog;
use crabomination::game::*;
use crabomination::mana::Color;
use crabomination::TurnStep;

/// Helper: flood a seat with plenty of every color + colorless mana.
fn flood_mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// Cast for the full {6} cost: a colorless 5/4 Construct.
#[test]
fn goring_warplow_full_cost_is_colorless_5_4() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::goring_warplow());
    flood_mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast full");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).expect("on battlefield");
    assert_eq!((cp.power, cp.toughness), (5, 4));
    assert!(cp.colors.is_empty(), "full-cost prototype is colorless");
    assert!(cp.keywords.contains(&Keyword::Deathtouch));
}

/// Cast for the prototype {1}{B} cost: a black 1/1 that keeps its abilities.
#[test]
fn goring_warplow_prototype_is_black_1_1_with_deathtouch() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::goring_warplow());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1); // {1}{B}
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast prototype");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).expect("on battlefield");
    assert_eq!((cp.power, cp.toughness), (1, 1), "prototype size");
    assert_eq!(cp.colors, vec![Color::Black], "prototype color follows its cost");
    assert!(cp.keywords.contains(&Keyword::Deathtouch), "keeps abilities");
    let r = g.battlefield_find(id).unwrap();
    assert!(r.cast_as_prototype);
    assert_eq!(r.definition.cost.cmc(), 2, "prototype mana value");
}

/// A prototype creature round-trips its smaller cost/color/size through a
/// name→factory snapshot (CR 702.160c copiable values).
#[test]
fn prototype_state_survives_snapshot_roundtrip() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::blitz_automaton());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2); // {2}{R}
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast prototype");
    drain_stack(&mut g);
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: GameState = serde_json::from_str(&json).expect("deserialize");
    let cp = g2.computed_permanent(id).expect("on battlefield after restore");
    assert_eq!((cp.power, cp.toughness), (3, 2));
    assert_eq!(cp.colors, vec![Color::Red]);
    assert!(cp.keywords.contains(&Keyword::Haste));
}

/// Combat Thresher's ETB draws a card regardless of cast mode.
#[test]
fn combat_thresher_prototype_draws_and_double_strikes() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::combat_thresher());
    let lib_id = g.next_id();
    g.players[0].library.push(crabomination::card::CardInstance::new(
        lib_id, catalog::goring_warplow(), 0,
    ));
    let before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast prototype");
    drain_stack(&mut g);
    // The spell left hand (−1) but the ETB drew a card (+1) → net same.
    assert_eq!(g.players[0].hand.len(), before);
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.keywords.contains(&Keyword::DoubleStrike));
}

/// Boulderbranch Golem gains life equal to its power on ETB — the prototype
/// face gains 3 (its 3/3), not the full 6.
#[test]
fn boulderbranch_golem_prototype_gains_three_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::boulderbranch_golem());
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3); // {3}{G}
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast prototype");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "gain life = prototype power 3");
}

/// Cradle Clearcutter taps for {G} equal to its power (prototype 1/3 → 1).
#[test]
fn cradle_clearcutter_taps_for_power_in_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::cradle_clearcutter());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == id) {
        c.summoning_sick = false;
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    // Full-cost body is a 3/6, so it taps for 3 green.
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3);
}

/// Phyrexian Fleshgorger's Ward—Pay life equal to its power: targeting the
/// full-cost 7/5 with an opponent's removal costs 7 life or the spell is
/// countered by Ward.
#[test]
fn fleshgorger_ward_costs_life_equal_to_power() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::phyrexian_fleshgorger());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 5));
    assert!(cp.keywords.contains(&Keyword::Ward(WardCost::LifeSourcePower)));
    assert!(cp.keywords.contains(&Keyword::Menace));
    assert!(cp.keywords.contains(&Keyword::Lifelink));
    // P1 tries to Shock it: Ward triggers, P1 must pay 7 life.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    // Ward auto-paid 7 life; the creature survives the 3-damage bolt (7 tough).
    assert_eq!(g.players[1].life, p1_life - 7, "Ward—pay 7 life (its power)");
    assert!(g.battlefield_find(id).is_some(), "Fleshgorger survives the bolt");
}

/// Frogmyr Enforcer's Affinity for artifacts reduces the prototype cost by
/// {1} per artifact controlled.
#[test]
fn frogmyr_enforcer_affinity_reduces_prototype_cost() {
    let mut g = two_player_game();
    // Two artifacts in play → affinity {2}.
    g.add_card_to_battlefield(0, catalog::goring_warplow());
    g.add_card_to_battlefield(0, catalog::blitz_automaton());
    let id = g.add_card_to_hand(0, catalog::frogmyr_enforcer());
    // Prototype {3}{R} − {2} affinity = {1}{R}.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("affinity-discounted prototype");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert_eq!(cp.colors, vec![Color::Red]);
}

/// Skitterbeam Battalion's ETB makes two token copies of itself (prototype
/// 2/2 face → two 2/2 tokens).
#[test]
fn skitterbeam_battalion_prototype_mints_two_copies() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::skitterbeam_battalion());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3); // {3}{R}{R}
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast prototype");
    drain_stack(&mut g);
    let battalions = g.battlefield.iter()
        .filter(|c| c.definition.name == "Skitterbeam Battalion" && c.controller == 0)
        .count();
    assert_eq!(battalions, 3, "original + two token copies");
}

/// Spotter Thopter's ETB scries X = its power; prototype 2/3 face flies.
#[test]
fn spotter_thopter_prototype_flies() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spotter_thopter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3); // {3}{U}
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast prototype");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Fallaji Dragon Engine pumps itself +1/+0 for {2}.
#[test]
fn fallaji_dragon_engine_firebreathes() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::fallaji_dragon_engine());
    let base = g.computed_permanent(id).unwrap().power;
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(id).unwrap().power, base + 1);
}

/// Autonomous Assembler puts a +1/+1 counter on a target Assembly-Worker.
#[test]
fn autonomous_assembler_counters_an_assembly_worker() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::autonomous_assembler());
    let tgt = g.add_card_to_battlefield(0, catalog::autonomous_assembler());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == src) { c.summoning_sick = false; }
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: src, ability_index: 0,
        target: Some(Target::Permanent(tgt)), additional_targets: vec![], x_value: None,
    }).expect("counter ability");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(tgt).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Iron-Craw Crusher's attack trigger pumps a target attacker by its power
/// (it auto-targets itself: full body 4/6 → +4 → 8/6).
#[test]
fn iron_craw_crusher_pumps_an_attacker() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let crusher = g.add_card_to_battlefield(0, catalog::iron_craw_crusher());
    g.clear_sickness(crusher);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: crusher, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(crusher).unwrap().power, 8, "pumped by its own power");
}

/// Steel Seraph grants a creature you control flying at the start of combat
/// (the ground creature is first in battlefield order, so it's auto-targeted).
#[test]
fn steel_seraph_grants_flying_at_combat() {
    let mut g = two_player_game();
    let ground = g.add_card_to_battlefield(0, catalog::goring_warplow());
    g.add_card_to_battlefield(0, catalog::steel_seraph());
    for c in g.battlefield.iter_mut() { c.summoning_sick = false; }
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(g.computed_permanent(ground).unwrap().keywords.contains(&Keyword::Flying));
}

/// CR 702.160c — a prototype permanent reverts to its printed (full,
/// colorless) characteristics when it leaves the battlefield.
#[test]
fn prototype_reverts_to_printed_when_it_dies() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::goring_warplow());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1); // {1}{B} prototype
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast prototype");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().definition.cost.cmc(), 2, "prototype MV on battlefield");
    // Send it to the graveyard; there it has its full printed cost/size.
    g.remove_to_graveyard_with_triggers(id);
    let dead = g.players[0].graveyard.iter().find(|c| c.id == id).expect("in graveyard");
    assert_eq!(dead.definition.cost.cmc(), 6, "full printed MV off the battlefield");
    assert_eq!((dead.definition.power, dead.definition.toughness), (5, 4));
    assert!(!dead.cast_as_prototype);
}

// ── BRO non-prototype cards ──────────────────────────────────────────────────

/// Diabolic Intent sacrifices a creature and tutors a chosen card to hand.
#[test]
fn diabolic_intent_sacrifices_and_tutors() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::goring_warplow());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));
    let id = g.add_card_to_hand(0, catalog::diabolic_intent());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Diabolic Intent");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "tutored card in hand");
}

/// Recommission reanimates a small creature with a +1/+1 counter.
#[test]
fn recommission_reanimates_with_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // 2/2, MV 2
    let id = g.add_card_to_hand(0, catalog::recommission());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Recommission");
    drain_stack(&mut g);
    let r = g.battlefield_find(bear).expect("reanimated");
    assert_eq!(r.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!((r.power(), r.toughness()), (3, 3), "2/2 + counter");
}

/// Depth Charge Colossus prototype is a 6/6 that doesn't untap but can be
/// untapped for {3}.
#[test]
fn depth_charge_colossus_doesnt_untap_then_untaps_for_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::depth_charge_colossus());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4); // {4}{U}{U}
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast prototype");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(id).map(|c| (c.power, c.toughness)), Some((6, 6)));
    // Tap it, run an untap step: it stays tapped.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == id) { c.tapped = true; }
    g.do_untap();
    assert!(g.battlefield_find(id).unwrap().tapped, "doesn't untap during untap step");
    // Pay {3} to untap.
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("untap ability");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(id).unwrap().tapped, "untapped for {{3}}");
}

/// Powerstone Shard taps for {C} per Powerstone Shard you control.
#[test]
fn powerstone_shard_scales_with_copies() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::powerstone_shard());
    g.add_card_to_battlefield(0, catalog::powerstone_shard());
    g.perform_action(GameAction::ActivateAbility {
        card_id: a, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 2, "one colorless per Shard controlled");
}

/// Bitter Reunion's sac ability grants your creatures haste.
#[test]
fn bitter_reunion_sac_grants_haste() {
    let mut g = two_player_game();
    let reunion = g.add_card_to_battlefield(0, catalog::bitter_reunion());
    let beater = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: reunion, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for haste");
    drain_stack(&mut g);
    assert!(g.battlefield_find(reunion).is_none(), "sacrificed");
    assert!(g.computed_permanent(beater).unwrap().keywords.contains(&Keyword::Haste));
}

/// Tocasia's Welcome draws once for a small creature entering, but only once
/// per turn.
#[test]
fn tocasias_welcome_draws_once_per_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tocasias_welcome());
    let nid = g.next_id();
    g.players[0].library.push(crabomination::card::CardInstance::new(nid, catalog::lightning_bolt(), 0));
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let cast_bear = |g: &mut GameState| {
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // MV 2 ≤ 3
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bear");
        drain_stack(g);
    };
    cast_bear(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "first small creature draws the bolt");
    cast_bear(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "only once each turn (no extra draw)");
}

/// Aeronaut Cavalry's ETB puts a +1/+1 counter on another Soldier you control.
#[test]
fn aeronaut_cavalry_counters_another_soldier() {
    let mut g = two_player_game();
    let other = g.add_card_to_battlefield(0, catalog::aeronaut_cavalry());
    let cav = g.add_card_to_hand(0, catalog::aeronaut_cavalry());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4); // {4}{W}
    g.perform_action(GameAction::CastSpell {
        card_id: cav, target: Some(Target::Permanent(other)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Aeronaut Cavalry");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Penregon Strongbull sacs an artifact to pump itself and ping each opponent.
#[test]
fn penregon_strongbull_sacs_artifact_for_pump_and_ping() {
    let mut g = two_player_game();
    let bull = g.add_card_to_battlefield(0, catalog::penregon_strongbull());
    let art = g.add_card_to_battlefield(0, catalog::powerstone_shard());
    g.clear_sickness(bull);
    g.players[0].mana_pool.add_colorless(1);
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bull, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact sacrificed");
    assert_eq!(g.computed_permanent(bull).unwrap().power, 3, "+1/+1");
    assert_eq!(g.players[1].life, opp_life - 1, "1 damage to each opponent");
}

/// Phyrexian Warhorse makes a Soldier only when kicked.
#[test]
fn phyrexian_warhorse_kicked_makes_a_soldier() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::phyrexian_warhorse());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::White, 1); // kicker {W}
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.definition.name == "Soldier" && c.controller == 0).count();
    assert_eq!(soldiers, 1, "kicked → one Soldier token");
}

/// Phyrexian Warhorse makes no token when cast unkicked.
#[test]
fn phyrexian_warhorse_unkicked_makes_no_soldier() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::phyrexian_warhorse());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3); // {3}{B}, no kicker
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast unkicked");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter().filter(|c| c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 0, "unkicked → no token");
}

/// The affordance probe surfaces a payable prototype cast.
#[test]
fn prototype_affordance_surfaced() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::rust_goliath());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3); // {3}{G}{G}
    let aff = g.compute_hand_affordances(0);
    assert!(aff.prototypable.contains(&id), "prototype cast offered when payable");
    // Full {10} cost isn't available, so the plain cast is not.
    assert!(!aff.castable.contains(&id));
}
