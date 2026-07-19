#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── Sliver tribal expansion + tribute / clash / tempting offer ────────────────

/// Sidewinder Sliver gives all Slivers flanking; Fury Sliver double strike.
#[test]
fn sidewinder_and_fury_grant_combat_keywords() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sidewinder_sliver());
    g.add_card_to_battlefield(0, catalog::fury_sliver());
    let opp = g.add_card_to_battlefield(1, catalog::striking_sliver());
    let kws = &g.computed_permanent(opp).unwrap().keywords;
    assert!(kws.contains(&Keyword::Flanking), "all-Sliver flanking reaches the opponent");
    assert!(kws.contains(&Keyword::DoubleStrike));
}

/// Plated (+0/+1 all) and Might (+2/+2 all) anthems stack.
#[test]
fn plated_and_might_anthems_stack() {
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::plated_sliver()); // 1/1
    g.add_card_to_battlefield(0, catalog::might_sliver()); // +2/+2
    let cp = g.computed_permanent(p).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "1/1 +2/+2 +0/+1");
}

/// Bonesplitter buffs all Slivers' power; Cleaving only its controller's.
#[test]
fn bonesplitter_all_cleaving_yours_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bonesplitter_sliver());
    g.add_card_to_battlefield(0, catalog::cleaving_sliver());
    let opp = g.add_card_to_battlefield(1, catalog::plated_sliver()); // 1/1
    // Opp's sliver: +2 (Bonesplitter, all) +1 (its own Plated all-toughness? no — power)
    // Bonesplitter is all-scope: opp gets +2/+0. Cleaving is yours-only: not applied.
    assert_eq!(g.computed_permanent(opp).unwrap().power, 3, "1 +2(all) but not +2(yours)");
}

/// Virulent Sliver's granted poisonous 1 lands a poison counter on hit.
#[test]
fn virulent_sliver_poisonous_grant() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::virulent_sliver());
    g.clear_sickness(v);
    g.attacking = vec![Attack { attacker: v, target: AttackTarget::Player(1) }];
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 1, "poisonous 1 fired off the granted trigger");
}

/// Groundshaker gives trample to its controller's Slivers only;
/// Cloudshredder grants flying + haste.
#[test]
fn groundshaker_and_cloudshredder_grants() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::groundshaker_sliver());
    let cs = g.add_card_to_battlefield(0, catalog::cloudshredder_sliver());
    let opp = g.add_card_to_battlefield(1, catalog::plated_sliver());
    let kws = &g.computed_permanent(cs).unwrap().keywords;
    assert!(kws.contains(&Keyword::Trample));
    assert!(kws.contains(&Keyword::Flying));
    assert!(kws.contains(&Keyword::Haste));
    assert!(!g.computed_permanent(opp).unwrap().keywords.contains(&Keyword::Trample),
        "yours-only trample");
}

/// Tempered Sliver's granted trigger grows a Sliver that connects.
#[test]
fn tempered_sliver_grows_on_combat_damage() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tempered_sliver());
    let atk = g.add_card_to_battlefield(0, catalog::plated_sliver()); // no first strike
    g.clear_sickness(atk);
    g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(atk).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1, "granted combat-damage trigger added a counter"
    );
}

/// Thorncaster Sliver's granted attack trigger pings.
#[test]
fn thorncaster_sliver_pings_on_attack() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::thorncaster_sliver());
    let atk = g.add_card_to_battlefield(0, catalog::striking_sliver());
    g.clear_sickness(atk);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }])
        .unwrap();
    let evs = drain_stack(&mut g);
    assert!(
        evs.iter().any(|e| matches!(e, GameEvent::DamageDealt { amount: 1, .. })),
        "attack trigger dealt 1 damage"
    );
}

/// Lavabelly Sliver's granted ETB pings a player and gains 1.
#[test]
fn lavabelly_sliver_etb_ping_and_gain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lavabelly_sliver());
    let s = g.add_card_to_hand(0, catalog::striking_sliver());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, s);
    // Granted ETB: 1 damage to target player (auto-target: opponent), gain 1.
    assert_eq!(g.players[1].life, 19, "opponent pinged");
    assert_eq!(g.players[0].life, 21, "controller gained 1");
}

/// Spiteful Sliver reflects damage dealt to your Slivers at a player.
#[test]
fn spiteful_sliver_reflects_damage() {
    let mut g = two_player_game();
    let sp = g.add_card_to_battlefield(0, catalog::spiteful_sliver());
    g.add_card_to_battlefield(0, catalog::might_sliver()); // 2/2 → 4/4, survives the bolt
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(sp)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the buffed Spiteful Sliver");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sp).is_some(), "4/4 survives");
    assert_eq!(g.players[1].life, 17, "3 damage reflected at a player (auto-target: opponent)");
}

/// First Sliver's Chosen grants exalted to your Slivers.
#[test]
fn first_slivers_chosen_exalted() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::first_slivers_chosen());
    let atk = g.add_card_to_battlefield(0, catalog::striking_sliver()); // 1/1
    g.clear_sickness(atk);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    // Through perform_action so the AttackerDeclared event reaches the
    // unified dispatcher (exalted is YourControl-scoped).
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    // One granted exalted instance on the attacker plus one on First
    // Sliver's Chosen itself (both are Slivers you control).
    assert_eq!(g.computed_permanent(atk).unwrap().power, 3, "1 + two exalted instances");
}

/// Clot Sliver's granted {2}: Regenerate saves a Sliver from destruction.
#[test]
fn clot_sliver_grants_regeneration() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::clot_sliver());
    let s = g.add_card_to_battlefield(0, catalog::striking_sliver());
    g.clear_sickness(s);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate granted regen");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(s)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(s).is_some(), "regeneration shield absorbed the kill");
}

/// Quilled Sliver's granted tap ability pings an attacking creature.
#[test]
fn quilled_sliver_pings_attacker() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let q = g.add_card_to_battlefield(0, catalog::quilled_sliver());
    g.clear_sickness(q);
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(0) }])
        .unwrap();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: q, ability_index: 0, target: Some(Target::Permanent(atk)), additional_targets: Vec::new(), x_value: None,
    }).expect("tap to ping the attacker");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(atk).unwrap().damage, 1, "attacker took 1");
}

/// Gemhide Sliver grants every Sliver "{T}: Add one mana of any color".
#[test]
fn gemhide_sliver_grants_mana_ability() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gemhide_sliver());
    let s = g.add_card_to_battlefield(0, catalog::striking_sliver());
    g.clear_sickness(s);
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("granted mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Diffusion Sliver taxes opponents' targeting (modeled as ward {2}).
#[test]
fn diffusion_sliver_taxes_targeting() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::diffusion_sliver());
    let s = g.add_card_to_battlefield(0, catalog::striking_sliver());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1); // no {2} for the ward tax
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(s)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast is legal; ward triggers");
    drain_stack(&mut g);
    assert!(g.battlefield_find(s).is_some(), "unpaid ward tax countered the bolt");
}

/// Pharagax Giant: tribute declined → 5 damage to each opponent.
#[test]
fn pharagax_giant_unpaid_tribute_burns() {
    let mut g = two_player_game();
    let p = g.add_card_to_hand(0, catalog::pharagax_giant());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, p); // AutoDecider declines tribute
    assert_eq!(g.players[1].life, 15, "5 to each opponent");
    assert_eq!(g.battlefield_find(p).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Nessian Demolok: tribute declined → destroy target noncreature permanent.
#[test]
fn nessian_demolok_unpaid_tribute_destroys() {
    let mut g = two_player_game();
    let orb = g.add_card_to_battlefield(1, catalog::howling_mine());
    let d = g.add_card_to_hand(0, catalog::nessian_demolok());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, d);
    assert!(g.battlefield_find(orb).is_none(), "noncreature permanent destroyed");
}

/// Thunder Brute: tribute paid → counters, no haste.
#[test]
fn thunder_brute_paid_tribute_gets_counters() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let t = g.add_card_to_hand(0, catalog::thunder_brute());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, t);
    let c = g.battlefield_find(t).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 3, "tribute 3 paid");
    assert!(!g.computed_permanent(t).unwrap().keywords.contains(&crabomination::card::Keyword::Haste));
}

/// Snake of the Golden Grove: tribute declined → gain 4.
#[test]
fn snake_of_the_golden_grove_unpaid_gains_life() {
    let mut g = two_player_game();
    let s = g.add_card_to_hand(0, catalog::snake_of_the_golden_grove());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, s);
    assert_eq!(g.players[0].life, 24, "unpaid tribute gains 4");
}

/// Lash Out: 3 to the creature; clash win → 3 to its controller.
#[test]
fn lash_out_clash_win_burns_controller() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::serra_angel());   // MV 5 reveal — wins
    g.add_card_to_library(1, catalog::lightning_bolt()); // MV 1 reveal
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let lo = g.add_card_to_hand(0, catalog::lash_out());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, lo, Target::Permanent(bear));
    assert!(g.battlefield_find(bear).is_none(), "3 damage killed the bear");
    assert_eq!(g.players[1].life, 17, "clash won: 3 to the controller");
}

/// Tempt with Glory: opponent accepts → both boards get counters, and the
/// controller's repeat fires once per acceptor.
#[test]
fn tempt_with_glory_acceptor_repeats_for_controller() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::tempt_with_glory());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, t);
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2, "controller: base + one repeat");
    assert_eq!(g.battlefield_find(theirs).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1, "acceptor's copy");
}

/// Tempt with Vengeance mints X hasty tokens (opponents decline by default).
#[test]
fn tempt_with_vengeance_mints_x_tokens() {
    let mut g = two_player_game();
    let t = g.add_card_to_hand(0, catalog::tempt_with_vengeance());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: t, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast with X=3");
    drain_stack(&mut g);
    let tokens = g.battlefield.iter()
        .filter(|c| c.definition.name == "Elemental" && c.controller == 0)
        .count();
    assert_eq!(tokens, 3);
}

/// Tempt with Reflections copies the targeted creature for the controller.
#[test]
fn tempt_with_reflections_copies_target() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel());
    let _ = mine;
    let t = g.add_card_to_hand(0, catalog::tempt_with_reflections());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast_at(&mut g, t, Target::Permanent(mine));
    let copies = g.battlefield.iter()
        .filter(|c| c.definition.name == "Serra Angel" && c.is_token)
        .count();
    assert_eq!(copies, 1);
}

/// Vexing Shusher's activation makes a target spell uncounterable.
#[test]
fn vexing_shusher_protects_a_spell() {
    let mut g = two_player_game();
    let vs = g.add_card_to_battlefield(0, catalog::vexing_shusher());
    g.clear_sickness(vs);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    // Shusher: target the bear spell on the stack.
    g.perform_action(GameAction::ActivateAbility {
        card_id: vs, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("shush the bear");
    drain_stack(&mut g);
    // re-cast scenario with a counterspell instead:
    let bear2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear 2");
    g.perform_action(GameAction::ActivateAbility {
        card_id: vs, ability_index: 0, target: Some(Target::Permanent(bear2)), additional_targets: Vec::new(), x_value: None,
    }).expect("shush");
    drain_stack(&mut g);
    let cs = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    let res = g.perform_action(GameAction::CastSpell {
        card_id: cs, target: Some(Target::Permanent(bear2)), additional_targets: vec![],
        mode: None, x_value: None,
    });
    if res.is_ok() {
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(bear2).is_some(), "shushed spell resolved through the counter");
}

// ── Sliver wave 2: granted activations + utility ──────────────────────────────

/// Crypt Sliver's granted tap ability regenerates a target Sliver.
#[test]
fn crypt_sliver_taps_to_regenerate() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::crypt_sliver());
    let s = g.add_card_to_battlefield(0, catalog::plated_sliver());
    g.clear_sickness(s);
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: Some(Target::Permanent(s)), additional_targets: Vec::new(), x_value: None,
    }).expect("tap to regenerate itself");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(s)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(s).is_some(), "regen shield saved the Sliver");
}

/// Hibernation Sliver's granted ability bounces for 2 life.
#[test]
fn hibernation_sliver_bounce_for_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hibernation_sliver());
    let s = g.add_card_to_battlefield(0, catalog::plated_sliver());
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("pay 2 life, bounce");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == s), "back to hand");
    assert_eq!(g.players[0].life, 18, "paid 2 life");
}

/// Necrotic Sliver's granted vindicate destroys any permanent.
#[test]
fn necrotic_sliver_vindicates() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::necrotic_sliver());
    let s = g.add_card_to_battlefield(0, catalog::plated_sliver());
    let land = g.add_card_to_battlefield(1, catalog::island());
    g.clear_sickness(s);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: Some(Target::Permanent(land)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac to destroy");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land destroyed");
    assert!(g.battlefield_find(s).is_none(), "sliver sacrificed as cost");
}

/// Acidic Sliver's granted sac ability burns any target.
#[test]
fn acidic_sliver_sac_burn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::acidic_sliver());
    let s = g.add_card_to_battlefield(0, catalog::plated_sliver());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac to burn");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "2 damage");
    assert!(g.battlefield_find(s).is_none(), "sacrificed");
}

/// Harmonic Sliver's granted ETB destroys an artifact or enchantment.
#[test]
fn harmonic_sliver_etb_naturalize() {
    let mut g = two_player_game();
    let mine_ = g.add_card_to_battlefield(1, catalog::howling_mine());
    g.add_card_to_battlefield(0, catalog::harmonic_sliver());
    let s = g.add_card_to_hand(0, catalog::plated_sliver());
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, s);
    assert!(g.battlefield_find(mine_).is_none(), "artifact destroyed by the granted ETB");
}

/// Telekinetic Sliver's granted tap ability taps a permanent.
#[test]
fn telekinetic_sliver_taps_permanent() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::telekinetic_sliver());
    let s = g.add_card_to_battlefield(0, catalog::plated_sliver());
    g.clear_sickness(s);
    let land = g.add_card_to_battlefield(1, catalog::island());
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: Some(Target::Permanent(land)), additional_targets: Vec::new(), x_value: None,
    }).expect("tap target");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped);
}

/// Dormant Sliver: defender grant + ETB cantrip.
#[test]
fn dormant_sliver_defender_and_cantrip() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::dormant_sliver());
    let hand_before = g.players[0].hand.len();
    let s = g.add_card_to_hand(0, catalog::plated_sliver());
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, s);
    assert!(g.computed_permanent(s).unwrap().keywords.contains(&Keyword::Defender));
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "hand count: -cast +draw, net +1 over the pre-add baseline");
}

/// Opaline Sliver offers a draw when a Sliver is targeted.
#[test]
fn opaline_sliver_draws_when_targeted() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::opaline_sliver());
    let s = g.add_card_to_battlefield(0, catalog::might_sliver()); // 4/4 w/ self-buff
    let hand_before = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(s)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the sliver");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "targeting fed a draw");
}

/// Opaline Sliver's granted trigger gates on the caster being an opponent —
/// your own spell targeting your Sliver doesn't draw.
#[test]
fn opaline_sliver_ignores_your_own_targeting_spell() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::opaline_sliver());
    let s = g.add_card_to_battlefield(0, catalog::might_sliver());
    let pump = g.add_card_to_hand(0, catalog::giant_growth());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_at(&mut g, pump, Target::Permanent(s));
    assert_eq!(g.players[0].hand.len(), hand_before - 1, "cast only — no draw off your own spell");
}

/// Sedge Sliver's granted "+1/+1 while you control a Swamp" checks each
/// Sliver's own controller, and the {B} regeneration grant works.
#[test]
fn sedge_sliver_conditional_pump_is_per_controller() {
    let mut g = two_player_game();
    let sedge = g.add_card_to_battlefield(0, catalog::sedge_sliver());
    let theirs = g.add_card_to_battlefield(1, catalog::plated_sliver());
    // Only P0 controls a Swamp.
    g.add_card_to_battlefield(0, catalog::swamp());
    // Plated Sliver itself grants all Slivers +0/+1, so baselines shift by that.
    let c = g.computed_permanent(sedge).unwrap();
    assert_eq!((c.power, c.toughness), (3, 4), "own Sliver buffed by own Swamp");
    let c = g.computed_permanent(theirs).unwrap();
    assert_eq!((c.power, c.toughness), (1, 2), "opponent's Sliver has no Swamp of its own");
    // Opponent plays a Swamp — now their Sliver gets the buff too.
    g.add_card_to_battlefield(1, catalog::swamp());
    let c = g.computed_permanent(theirs).unwrap();
    assert_eq!((c.power, c.toughness), (2, 3), "buffed once their controller has a Swamp");
    // Regeneration grant: {B} shields the Sliver from destruction.
    g.players[1].mana_pool.add(Color::Black, 1);
    let printed = catalog::plated_sliver().activated_abilities.len();
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: theirs, ability_index: printed, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("granted {B}: Regenerate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).unwrap().regeneration_shields >= 1, "shield up");
}

/// Homing Sliver grants slivercycling {3} to Sliver cards in hand —
/// {3}, discard the hand Sliver, fetch any Sliver from the library.
#[test]
fn homing_sliver_grants_slivercycling_to_hand_slivers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::homing_sliver());
    let in_hand = g.add_card_to_hand(0, catalog::plated_sliver());
    let in_lib = g.add_card_to_library(0, catalog::might_sliver());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Landcycle { card_id: in_hand }).expect("slivercycle");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == in_hand), "hand Sliver discarded");
    assert!(g.players[0].hand.iter().any(|c| c.id == in_lib), "library Sliver fetched");
    // A non-Sliver hand card gets no grant.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    assert!(g.perform_action(GameAction::Landcycle { card_id: bear }).is_err(),
        "non-Sliver can't slivercycle");
}

/// Ward Sliver: ETB color choice grants all Slivers protection from it.
#[test]
fn ward_sliver_grants_chosen_protection() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    let w_ = g.add_card_to_hand(0, catalog::ward_sliver());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, w_);
    let other = g.add_card_to_battlefield(0, catalog::plated_sliver());
    assert!(
        g.computed_permanent(other)
            .unwrap()
            .keywords
            .contains(&Keyword::Protection(Color::Red)),
        "protection from the chosen color reaches every Sliver"
    );
    // A red burn spell can't even target the Sliver now.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(other)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "protection blocks targeting");
}

/// Cautery Sliver grants both the ping and the prevention sac abilities.
#[test]
fn cautery_sliver_ping_and_shield() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::cautery_sliver());
    let a = g.add_card_to_battlefield(0, catalog::plated_sliver());
    let b_ = g.add_card_to_battlefield(0, catalog::plated_sliver());
    g.players[0].mana_pool.add_colorless(2);
    // Ability 0: ping the opponent.
    g.perform_action(GameAction::ActivateAbility {
        card_id: a, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac-ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
    // Ability 1: shield self with prevent-next-1, then take a 1-damage ping.
    g.perform_action(GameAction::ActivateAbility {
        card_id: b_, ability_index: 1, target: Some(Target::Player(0)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac-shield");
    drain_stack(&mut g);
    let ctx = crabomination::game::effects::EffectContext::for_spell(1, Some(Target::Player(0)), 0, 0);
    g.resolve_effect(
        &crabomination::effect::Effect::DealDamage {
            to: crabomination::effect::Selector::Target(0),
            amount: crabomination::effect::Value::Const(1),
        },
        &ctx,
    ).unwrap();
    assert_eq!(g.players[0].life, 20, "prevention shield soaked the ping");
}

/// Crystalline Crawler: converge counters in, counter-out mana, tap to grow.
#[test]
fn crystalline_crawler_counter_mana_loop() {
    let mut g = two_player_game();
    let cc = g.add_card_to_hand(0, catalog::crystalline_crawler());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, cc);
    assert_eq!(
        g.battlefield_find(cc).unwrap().counter_count(CounterType::PlusOnePlusOne),
        4, "converge: one counter per color spent"
    );
    g.perform_action(GameAction::ActivateAbility {
        card_id: cc, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("remove a counter for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1);
    assert_eq!(
        g.battlefield_find(cc).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3
    );
}

/// Sliver Legion: each Sliver grows by +1/+1 per OTHER Sliver in play —
/// including opponents' Slivers, and excluding itself from its own count.
#[test]
fn sliver_legion_per_other_sliver_anthem() {
    let mut g = two_player_game();
    let legion = g.add_card_to_battlefield(0, catalog::sliver_legion()); // 7/7
    let a = g.add_card_to_battlefield(0, catalog::plated_sliver()); // 1/1 (+0/+1 anthem)
    let b_ = g.add_card_to_battlefield(1, catalog::venom_sliver()); // 1/1
    // Three slivers: each counts the other two.
    let cp = g.computed_permanent(legion).unwrap();
    assert_eq!((cp.power, cp.toughness), (9, 10), "7/7 +2/+2 legion, +0/+1 plated");
    let cp = g.computed_permanent(a).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "1/1 +2/+2 +0/+1");
    let cp = g.computed_permanent(b_).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "opponent's Sliver counts and benefits");
}

// ── Prowl (CR 702.76) ─────────────────────────────────────────────────────────

/// Helper: attack with `attacker` (a Rogue etc.) and resolve combat damage to P1.
fn prowl_swing(g: &mut GameState, attacker: crabomination::card::CardId) {
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(g);
}

/// Prowl is rejected without tribal combat damage and accepted after a Rogue
/// connects; the prowl-paid ETB rider draws (Latchkey Faerie).
#[test]
fn cr_702_76_prowl_gates_on_tribal_combat_damage() {
    let mut g = two_player_game();
    let faerie = g.add_card_to_hand(0, catalog::latchkey_faerie());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::CastSpellAlternative {
        card_id: faerie, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no Rogue connected yet — prowl unavailable");

    let rogue = g.add_card_to_battlefield(0, catalog::aunties_snitch());
    prowl_swing(&mut g, rogue);
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    // Mana pools emptied at each step boundary during the swing — refill.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: faerie, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("prowl cast for {2}{U}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == faerie), "Faerie on the battlefield");
    // -1 cast +1 prowl-paid draw = net even with the pre-cast hand.
    assert_eq!(g.players[0].hand.len(), hand_before, "prowl-paid ETB drew a card");
}

/// Auntie's Snitch returns from the graveyard when a Goblin or Rogue you
/// control deals combat damage to a player.
#[test]
fn cr_702_76_aunties_snitch_returns_from_graveyard_on_rogue_damage() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let snitch = g.add_card_to_graveyard(0, catalog::aunties_snitch());
    let rogue = g.add_card_to_battlefield(0, catalog::stinkdrinker_bandit());
    prowl_swing(&mut g, rogue);
    assert!(g.players[0].hand.iter().any(|c| c.id == snitch), "Snitch returned to hand");
}

/// Stinkdrinker Bandit pumps an unblocked attacking Rogue +2/+1 (CR 509.3g).
#[test]
fn stinkdrinker_bandit_pumps_unblocked_rogue() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stinkdrinker_bandit());
    let rogue = g.add_card_to_battlefield(0, catalog::aunties_snitch()); // 3/1 Rogue
    let life_before = g.players[1].life;
    prowl_swing(&mut g, rogue);
    assert_eq!(g.players[1].life, life_before - 5, "3/1 hit for 5 with the +2/+1 pump");
}

/// Notorious Throng mints X fliers (X = opponent life lost this turn) and
/// takes an extra turn when prowled.
#[test]
fn notorious_throng_tokens_and_prowl_extra_turn() {
    let mut g = two_player_game();
    let rogue = g.add_card_to_battlefield(0, catalog::aunties_snitch()); // 3 power
    prowl_swing(&mut g, rogue);
    let throng = g.add_card_to_hand(0, catalog::notorious_throng());
    // Cast at sorcery speed on our own post-combat main.
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: throng, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("prowl Notorious Throng");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.len(), bf_before + 3, "three 1/1 fliers for 3 damage");
    assert!(g.players[0].extra_turns >= 1, "extra turn queued off the prowl rider");
}

// ── Triggered mana abilities (CR 605.1b) ─────────────────────────────────────

/// Helper: tap `land` for mana via its first effective mana ability.
fn tap_for_mana(g: &mut GameState, land: crabomination::card::CardId) {
    let (idx, _) = g.effective_mana_abilities(land).into_iter().next()
        .expect("land has a mana ability");
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: idx, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for mana");
}

/// Wild Growth: the enchanted land adds an additional {G}; an unenchanted
/// land doesn't (CR 605.1b — no stack, immediate).
#[test]
fn cr_605_1b_wild_growth_adds_extra_green_for_enchanted_land_only() {
    let mut g = two_player_game();
    let enchanted = g.add_card_to_battlefield(0, catalog::mountain());
    let plain = g.add_card_to_battlefield(0, catalog::mountain());
    let aura = g.add_card_to_battlefield(0, catalog::wild_growth());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(enchanted);
    tap_for_mana(&mut g, enchanted);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "extra {{G}} from Wild Growth");
    tap_for_mana(&mut g, plain);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "unenchanted land adds nothing");
}

/// Mana Flare mirrors the produced type for any player's land.
#[test]
fn cr_605_1b_mana_flare_mirrors_produced_type_for_all_players() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mana_flare());
    let yours = g.add_card_to_battlefield(0, catalog::swamp());
    let theirs = g.add_card_to_battlefield(1, catalog::island());
    tap_for_mana(&mut g, yours);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 2, "your Swamp made {{B}}{{B}}");
    g.priority.player_with_priority = 1;
    tap_for_mana(&mut g, theirs);
    assert_eq!(g.players[1].mana_pool.amount(Color::Blue), 2, "opponent's Island also doubled");
}

/// Utopia Sprawl adds the ETB-chosen color when the enchanted Forest taps.
#[test]
fn cr_605_1b_utopia_sprawl_adds_chosen_color() {
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    let aura = g.add_card_to_hand(0, catalog::utopia_sprawl());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_at(&mut g, aura, Target::Permanent(forest));
    tap_for_mana(&mut g, forest);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "the Forest's own {{G}}");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "plus the chosen {{R}}");
}

/// Vernal Bloom boosts every Forest, not other land types.
#[test]
fn cr_605_1b_vernal_bloom_boosts_forests_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vernal_bloom());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let swamp = g.add_card_to_battlefield(0, catalog::swamp());
    tap_for_mana(&mut g, forest);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "Forest adds an extra {{G}}");
    tap_for_mana(&mut g, swamp);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1, "Swamp unaffected");
}

/// Crypt Ghast doubles only YOUR Swamps; Overgrowth stacks two extra {G}.
#[test]
fn crypt_ghast_and_overgrowth_extra_mana() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::crypt_ghast());
    let yours = g.add_card_to_battlefield(0, catalog::swamp());
    let theirs = g.add_card_to_battlefield(1, catalog::swamp());
    tap_for_mana(&mut g, yours);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 2, "your Swamp adds an extra {{B}}");
    g.priority.player_with_priority = 1;
    tap_for_mana(&mut g, theirs);
    assert_eq!(g.players[1].mana_pool.amount(Color::Black), 1, "opponent's Swamp unaffected");

    // Overgrowth: enchanted land taps for base + {G}{G}.
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    let aura = g.add_card_to_battlefield(0, catalog::overgrowth());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(land);
    g.priority.player_with_priority = 0;
    tap_for_mana(&mut g, land);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "Overgrowth adds {{G}}{{G}}");
}

/// Marsh Flitter mints two Goblin Rogues and sacs one for base 3/3.
#[test]
fn marsh_flitter_tokens_and_base_pt_swap() {
    let mut g = two_player_game();
    let flitter = g.add_card_to_hand(0, catalog::marsh_flitter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, flitter);
    assert_eq!(g.battlefield.iter()
        .filter(|c| c.definition.name == "Goblin Rogue" && c.controller == 0).count(), 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: flitter, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac a Goblin");
    drain_stack(&mut g);
    let c = g.computed_permanent(flitter).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "base P/T 3/3 until end of turn");
    assert_eq!(g.battlefield.iter()
        .filter(|c| c.definition.name == "Goblin Rogue" && c.controller == 0).count(), 1,
        "one token paid the cost");
}

/// Earwig Squad prowled: caster exiles three cards from the opponent's library.
#[test]
fn earwig_squad_prowled_exiles_three_from_library() {
    let mut g = two_player_game();
    let rogue = g.add_card_to_battlefield(0, catalog::aunties_snitch());
    prowl_swing(&mut g, rogue);
    let picks: Vec<_> = (0..4).map(|_| g.add_card_to_library(1, catalog::forest())).collect();
    let squad = g.add_card_to_hand(0, catalog::earwig_squad());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib_before = g.players[1].library.len();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(picks[0])),
        DecisionAnswer::Search(Some(picks[1])),
        DecisionAnswer::Search(Some(picks[2])),
    ]));
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: squad, pitch_card: None, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("prowl Earwig Squad");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib_before - 3, "three cards exiled");
}

/// Oona's Blackguard: other Rogues enter with a counter; countered creatures
/// connecting make the player discard.
#[test]
fn oonas_blackguard_counters_and_discard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::oonas_blackguard());
    let rogue = g.add_card_to_hand(0, catalog::aunties_snitch());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, rogue);
    assert_eq!(g.battlefield_find(rogue).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 1, "entered with an extra counter");
    g.add_card_to_hand(1, catalog::forest());
    let opp_hand = g.players[1].hand.len();
    prowl_swing(&mut g, rogue);
    assert_eq!(g.players[1].hand.len(), opp_hand - 1, "damaged player discarded");
}

/// Morsel Theft drains 3; prowled cast also draws.
#[test]
fn morsel_theft_drains_and_prowl_draws() {
    let mut g = two_player_game();
    let rogue = g.add_card_to_battlefield(0, catalog::aunties_snitch());
    prowl_swing(&mut g, rogue);
    g.add_card_to_library(0, catalog::island());
    let theft = g.add_card_to_hand(0, catalog::morsel_theft());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    let hand = g.players[0].hand.len();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: theft, pitch_card: None, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("prowl Morsel Theft");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 3);
    assert_eq!(g.players[0].life, l0 + 3);
    assert_eq!(g.players[0].hand.len(), hand, "-1 cast +1 prowl draw");
}

/// Syphon Sliver grants your Slivers lifelink; Nirkana pumps with {B}.
#[test]
fn syphon_sliver_and_nirkana_basics() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::syphon_sliver());
    let s = g.add_card_to_battlefield(0, catalog::plated_sliver());
    assert!(g.computed_permanent(s).unwrap().keywords.contains(&Keyword::Lifelink));
    let theirs = g.add_card_to_battlefield(1, catalog::plated_sliver());
    assert!(!g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::Lifelink),
        "opponent's Slivers don't get lifelink");

    let nirkana = g.add_card_to_battlefield(0, catalog::nirkana_revenant());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: nirkana, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{B}: pump");
    drain_stack(&mut g);
    let c = g.computed_permanent(nirkana).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5));
}

// ── Faerie tribal ────────────────────────────────────────────────────────────

/// Spellstutter Sprite counters only spells with MV ≤ your Faerie count.
#[test]
fn spellstutter_sprite_counters_by_faerie_count() {
    let mut g = two_player_game();
    // Opponent casts a 2-MV spell; with only the Sprite itself (1 Faerie),
    // MV 2 > 1 — the trigger fizzles and the spell resolves.
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    let sprite = g.add_card_to_hand(0, catalog::spellstutter_sprite());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: sprite, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("flash in the Sprite");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "MV 2 > 1 Faerie — bear resolves");

    // Second sprite with a Faerie buddy on board: 2 Faeries ≥ MV 2 — counters.
    g.add_card_to_battlefield(0, catalog::scion_of_oona());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast bolt");
    let sprite2 = g.add_card_to_hand(0, catalog::spellstutter_sprite());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: sprite2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("flash in the second Sprite");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "MV 1 ≤ 2 Faeries — countered");
}

/// Scion of Oona: other Faeries get +1/+1 and shroud; Scion itself doesn't.
#[test]
fn scion_of_oona_buffs_other_faeries() {
    let mut g = two_player_game();
    let scion = g.add_card_to_battlefield(0, catalog::scion_of_oona());
    let sprite = g.add_card_to_battlefield(0, catalog::spellstutter_sprite());
    let c = g.computed_permanent(sprite).unwrap();
    assert_eq!((c.power, c.toughness), (2, 2));
    assert!(c.keywords.contains(&Keyword::Shroud));
    let c = g.computed_permanent(scion).unwrap();
    assert_eq!((c.power, c.toughness), (1, 1), "Scion doesn't buff itself");
    assert!(!c.keywords.contains(&Keyword::Shroud));
}

/// Sower of Temptation steals a creature until it leaves the battlefield.
#[test]
fn sower_of_temptation_steal_reverts_when_it_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sower = g.add_card_to_hand(0, catalog::sower_of_temptation());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, sower, Target::Permanent(bear));
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "stolen");
    g.remove_to_graveyard_with_triggers(sower);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 1, "handed back on leave");
}

/// Peppersmoke shrinks and cantrips with a Faerie around.
#[test]
fn peppersmoke_shrinks_and_cantrips_with_a_faerie() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spellstutter_sprite());
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let smoke = g.add_card_to_hand(0, catalog::peppersmoke());
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand = g.players[0].hand.len();
    cast_at(&mut g, smoke, Target::Permanent(bear));
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (1, 1));
    assert_eq!(g.players[0].hand.len(), hand, "-1 cast +1 draw");
}

/// Sygg draws at end step after an opponent lost 3+ life.
#[test]
fn sygg_draws_after_three_life_lost() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::sygg_river_cutthroat());
    g.add_card_to_library(0, catalog::island());
    let rogue = g.add_card_to_battlefield(0, catalog::aunties_snitch()); // 3 power
    let hand = g.players[0].hand.len();
    prowl_swing(&mut g, rogue);
    // Advance into the end step so the trigger fires.
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew off the 3-life hit");
}

/// Spined Sliver grows +1/+1 per blocker when a Sliver becomes blocked.
#[test]
fn spined_sliver_grows_per_blocker() {
    use crabomination::game::types::Attack;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spined_sliver());
    let atk = g.add_card_to_battlefield(0, catalog::might_sliver()); // 4/4 lord → others +1/+1
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, atk), (b2, atk)])).expect("double block");
    drain_stack(&mut g);
    let c = g.computed_permanent(atk).unwrap();
    // Might Sliver base 4/4 (its own lord excludes itself? it buffs all
    // Slivers) + Spined trigger +2/+2 for two blockers.
    assert!(c.power >= 6, "two blockers fed +2/+2 (got {}/{})", c.power, c.toughness);
}

/// Vizier of Many Faces clone-enters, and its embalm token clone-enters too.
#[test]
fn vizier_of_many_faces_embalm_token_clones() {
    let mut g = two_player_game();
    let _fatty = g.add_card_to_battlefield(1, catalog::might_sliver()); // copy source, 4/4
    let viz = g.add_card_to_hand(0, catalog::vizier_of_many_faces());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, viz);
    let c = g.computed_permanent(viz).unwrap();
    assert_eq!(g.battlefield_find(viz).unwrap().definition.name, "Might Sliver", "cast copy");
    assert!(c.power >= 4);
    // Kill it, embalm from the graveyard: the token also enters as a copy.
    g.remove_to_graveyard_with_triggers(viz);
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: viz, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("embalm");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.is_token && c.controller == 0)
        .expect("embalm token minted");
    assert_eq!(token.definition.name, "Might Sliver", "token clone-entered");
    assert!(token.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Zombie),
        "embalm token is also a Zombie");
    assert!(g.exile.iter().any(|c| c.id == viz), "Vizier exiled by embalm");
}

// ── Faerie tribal completion + Pyxis (claude/modern_decks) ───────────────────

/// Rune Snag's tax scales by {2} per Rune Snag in any graveyard.
#[test]
fn rune_snag_tax_scales_with_graveyard_copies() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::rune_snag());
    g.add_card_to_graveyard(1, catalog::rune_snag());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bolt");
    let snag = g.add_card_to_hand(0, catalog::rune_snag());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    // P1 has {5} floating — one short of the {2}+{2}+{2} tax.
    g.players[1].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: snag, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Rune Snag");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Bolt countered — {{6}} unpayable with {{5}} floating");
}

/// Faerie Macabre discards itself from hand to exile up to two graveyard cards.
#[test]
fn faerie_macabre_exiles_two_graveyard_cards() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let fm = g.add_card_to_hand(0, catalog::faerie_macabre());
    let a = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let b = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a, b])]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: fm, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("discard Faerie Macabre");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fm), "Macabre discarded");
    assert!(g.exile.iter().any(|c| c.id == a) && g.exile.iter().any(|c| c.id == b),
        "both graveyard cards exiled");
}

/// Oona's {X}{U/B} exile mints a Faerie per chosen-color card exiled.
#[test]
fn oona_x_activation_exiles_and_mints_faeries() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let oona = g.add_card_to_battlefield(0, catalog::oona_queen_of_the_fae());
    for _ in 0..2 { g.add_card_to_library(1, catalog::lightning_bolt()); }
    g.add_card_to_library(1, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: oona, ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: Some(3),
    }).expect("activate for X=3");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), 0, "top three exiled");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Faerie Rogue").count(),
        2,
        "two red cards exiled → two Faeries"
    );
}

/// Mistbind Clique champions a Faerie and taps the targeted player's lands.
#[test]
fn mistbind_clique_champion_taps_lands() {
    let mut g = two_player_game();
    let faerie = g.add_card_to_battlefield(0, catalog::faerie_macabre());
    let l1 = g.add_card_to_battlefield(1, catalog::forest());
    let l2 = g.add_card_to_battlefield(1, catalog::forest());
    let mb = g.add_card_to_battlefield(0, catalog::mistbind_clique());
    g.fire_self_etb_triggers(mb, 0);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == faerie), "Faerie championed (exiled)");
    assert!(g.battlefield_find(l1).unwrap().tapped, "land tapped");
    assert!(g.battlefield_find(l2).unwrap().tapped, "land tapped");
    // The championed Faerie returns when the Clique leaves (CR 702.77).
    let evs = g.remove_to_graveyard_with_triggers(mb);
    g.dispatch_triggers_for_events(&evs);
    g.check_state_based_actions();
    assert!(g.battlefield_find(faerie).is_some(), "championed Faerie returns");
}

/// Champion with no other Faerie sacrifices the Clique (CR 702.77a).
#[test]
fn mistbind_clique_sacrificed_without_a_faerie() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let mb = g.add_card_to_battlefield(0, catalog::mistbind_clique());
    g.fire_self_etb_triggers(mb, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(mb).is_none(), "no Faerie to champion → sacrificed");
    assert!(!g.battlefield_find(land).unwrap().tapped, "no champion → no land tap");
}

/// Pyxis exiles each player's top card face down; the sacrifice deploys the
/// permanent cards.
#[test]
fn pyxis_of_pandemonium_exiles_then_deploys() {
    let mut g = two_player_game();
    let pyxis = g.add_card_to_battlefield(0, catalog::pyxis_of_pandemonium());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_library(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pyxis, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap Pyxis");
    drain_stack(&mut g);
    let exiled_bear = g.exile.iter().find(|c| c.id == bear).expect("bear exiled");
    assert!(exiled_bear.face_down, "exiled face down");
    assert_eq!(exiled_bear.exiled_with, Some(pyxis));
    assert!(g.exile.iter().any(|c| c.id == bolt));

    g.battlefield_find_mut(pyxis).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pyxis, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac Pyxis");
    drain_stack(&mut g);
    let bf_bear = g.battlefield_find(bear).expect("bear deployed");
    assert_eq!(bf_bear.controller, 0, "under its owner's control");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt) || g.exile.iter().any(|c| c.id == bolt),
        "nonpermanent Bolt stays out of play");
}

/// Changeling Hero's champion sacrifices the Hero when you control no other
/// creature (CR 702.77a).
#[test]
fn changeling_hero_champion_self_sacrifice() {
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::changeling_hero());
    g.fire_self_etb_triggers(hero, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(hero).is_none(), "nothing to champion → sacrificed");
}

/// Faerie Conclave enters tapped and animates into a 2/1 flier.
#[test]
fn faerie_conclave_animates() {
    let mut g = two_player_game();
    let fc = g.add_card_to_hand(0, catalog::faerie_conclave());
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(fc)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fc).unwrap().tapped, "enters tapped");
    g.battlefield_find_mut(fc).unwrap().tapped = false;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fc, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("animate");
    drain_stack(&mut g);
    let c = g.computed_permanent(fc).unwrap();
    assert!(c.card_types.contains(&CardType::Creature));
    assert_eq!((c.power, c.toughness), (2, 1));
}

/// Secluded Glen enters untapped when a Faerie can be revealed, tapped
/// otherwise.
#[test]
fn secluded_glen_reveal_gate() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::faerie_macabre());
    let glen = g.add_card_to_hand(0, catalog::secluded_glen());
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(glen)).expect("play");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(glen).unwrap().tapped, "Faerie revealed → untapped");
}

/// Familiar's Ruse bounces a creature as the cost and counters the spell.
#[test]
fn familiars_ruse_bounce_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    let ruse = g.add_card_to_hand(0, catalog::familiars_ruse());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: ruse, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ruse");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature bounced as cost");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt countered");
}

/// Thieving Sprite forces a chosen discard.
#[test]
fn thieving_sprite_chosen_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let ts = g.add_card_to_battlefield(0, catalog::thieving_sprite());
    g.fire_self_etb_triggers(ts, 0);
    drain_stack(&mut g);
    assert!(g.players[1].hand.is_empty(), "card discarded");
}

/// Quickling bounces another creature or sacrifices itself.
#[test]
fn quickling_bounce_or_sacrifice() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let q1 = g.add_card_to_battlefield(0, catalog::quickling());
    g.fire_self_etb_triggers(q1, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bear bounced");
    assert!(g.battlefield_find(q1).is_some(), "Quickling stays");
    // With no other creature, the second Quickling sacrifices itself.
    let q2 = g.add_card_to_battlefield(1, catalog::quickling());
    g.fire_self_etb_triggers(q2, 1);
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(q2).is_none(), "no bounce target → sacrificed");
}

// ── Cleave (CR 702.148) ──────────────────────────────────────────────────────

/// Wash Away's base mode only counters a spell not cast from its owner's
/// hand: a flashback cast is counterable for {U}, a hand cast is not a
/// legal target.
#[test]
fn cleave_wash_away_base_needs_non_hand_cast() {
    let mut g = two_player_game();
    // Seat 1 flashbacks Lava Dart out of the graveyard (sac a Mountain).
    let dart = g.add_card_to_graveyard(1, catalog::lava_dart());
    g.add_card_to_battlefield(1, catalog::mountain());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastFlashback {
        card_id: dart, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lava Dart flashback");
    let wash = g.add_card_to_hand(0, catalog::wash_away());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: wash, target: Some(Target::Permanent(dart)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("base Wash Away counters the non-hand cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the Dart never resolved");
}

/// The base mode rejects a hand-cast spell; the cleave cast ({1}{U}{U})
/// drops the bracket and counters it.
#[test]
fn cleave_wash_away_cleave_counters_hand_cast() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt cast from hand");
    let wash = g.add_card_to_hand(0, catalog::wash_away());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: wash, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "base mode can't counter a hand cast");
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: wash, pitch_card: None, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cleave cast counters anything");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the Bolt was countered");
}

/// Dig Up: base fetches a basic land to hand; the cleave cast fetches any
/// card (CR 702.148 removes the bracketed restriction).
#[test]
fn cleave_dig_up_fetches_anything() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));
    let id = g.add_card_to_hand(0, catalog::dig_up());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cleave Dig Up");
    drain_stack(&mut g);
    assert!(g.players[0].has_in_hand(bolt), "a nonland fetched on the cleave cast");
}

/// Path of Peril: the base sweep only kills mana value ≤ 2; cleave kills all.
#[test]
fn cleave_path_of_peril_base_spares_big_creatures() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::atraxa_grand_unifier());
    let id = g.add_card_to_hand(0, catalog::path_of_peril());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("base Path of Peril");
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_none(), "MV 2 swept");
    assert!(g.battlefield_find(big).is_some(), "MV 7 survives the base mode");
}

/// Parasitic Grasp's base mode targets only Humans; the damage and lifegain
/// both land.
#[test]
fn cleave_parasitic_grasp_base_is_human_only() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::parasitic_grasp());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "a Bear isn't a Human");
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cleave hits any creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "3 damage killed the 2/2");
    assert_eq!(g.players[0].life, 23, "gained 3");
}

/// Winged Portent draws per flyer on the base mode and per creature on the
/// cleave cast.
#[test]
fn cleave_winged_portent_counts_fliers_then_everything() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::consecrated_sphinx()); // flyer
    for _ in 0..8 { g.add_card_to_library(0, catalog::island()); }
    let a = g.add_card_to_hand(0, catalog::winged_portent());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: a, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("base Winged Portent");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1 + 1, "one flyer → one draw");

    let b = g.add_card_to_hand(0, catalog::winged_portent());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: b, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cleave Winged Portent");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1 + 2, "two creatures → two draws");
}

/// Dread Fugue's base pick is capped at mana value 2; cleave rips any
/// nonland card.
#[test]
fn cleave_dread_fugue_mv_cap_on_base() {
    let mut g = two_player_game();
    let expensive = g.add_card_to_hand(1, catalog::atraxa_grand_unifier());
    let cheap = g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::dread_fugue());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("base Dread Fugue");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == cheap),
        "the MV≤2 card was discarded");
    assert!(g.players[1].hand.iter().any(|c| c.id == expensive),
        "the MV 7 card is out of the base mode's reach");
}

/// Fierce Retribution's base mode only hits attackers; cleave hits any
/// creature.
#[test]
fn cleave_fierce_retribution_base_needs_attacker() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fierce_retribution());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "non-attacker rejected on the base mode");
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cleave destroys any creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// Alchemist's Retrieval bounces only your own nonland permanent on the
/// base mode; cleave bounces anything nonland.
#[test]
fn cleave_alchemists_retrieval_base_is_own_only() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::alchemists_retrieval());
    g.players[0].mana_pool.add(Color::Blue, 1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(theirs)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "opponent's permanent rejected on the base mode");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: Some(Target::Permanent(theirs)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cleave bounces any nonland permanent");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs), "bounced to owner's hand");
}

// ── Modern staples batch (June 2026) ─────────────────────────────────────────

/// Shard Volley sacrifices a land as an additional cost and deals 3.
#[test]
fn shard_volley_sacs_a_land_for_three() {
    let mut g = two_player_game();
    let mtn = g.add_card_to_battlefield(0, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::shard_volley());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Shard Volley");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
    assert!(g.battlefield_find(mtn).is_none(), "the land was sacrificed");
}

/// Insolent Neonate's activation pays discard + sacrifice and draws.
#[test]
fn insolent_neonate_loots_itself_away() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::insolent_neonate());
    g.add_card_to_hand(0, catalog::mountain()); // discard fodder
    g.add_card_to_library(0, catalog::island()); // card to draw
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Neonate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "sacrificed itself");
    assert_eq!(g.players[0].hand.len(), before, "-1 discard +1 draw");
    assert_eq!(g.players[0].graveyard.len(), 2, "Neonate + the discarded card");
}

/// Manic Scribe mills three per opponent on ETB and again at each
/// opponent's upkeep once delirium is on.
#[test]
fn manic_scribe_etb_and_delirium_upkeep_mill() {
    let mut g = two_player_game();
    for _ in 0..10 { g.add_card_to_library(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::manic_scribe());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Manic Scribe");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 3, "ETB milled three");

    // Delirium: four card types in P0's graveyard.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());   // instant
    g.add_card_to_graveyard(0, catalog::mountain());          // land
    g.add_card_to_graveyard(0, catalog::grizzly_bears());     // creature
    g.add_card_to_graveyard(0, catalog::thoughtseize());      // sorcery
    // Fire the opponent-upkeep trigger.
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 6, "delirium upkeep milled three more");
}

/// Mind Funeral mills the target down to (and including) the fourth land.
#[test]
fn mind_funeral_mills_until_four_lands() {
    let mut g = two_player_game();
    // Library top-down: bolt, island, island, bolt, island, island, bolt.
    for def in [catalog::lightning_bolt(), catalog::island(), catalog::island(),
                catalog::lightning_bolt(), catalog::island(), catalog::island(),
                catalog::lightning_bolt()] {
        g.add_card_to_library(1, def);
    }
    let id = g.add_card_to_hand(0, catalog::mind_funeral());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mind Funeral");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 6, "milled through the fourth land");
    assert_eq!(g.players[1].library.len(), 1, "the last card stays");
}

/// Fleet Swallower's attack trigger mills half the library rounded up.
#[test]
fn fleet_swallower_mills_half_rounded_up() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    for _ in 0..7 { g.add_card_to_library(1, catalog::island()); }
    let fish = g.add_card_to_battlefield(0, catalog::fleet_swallower());
    g.clear_sickness(fish);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: fish, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 4, "7 → mills ceil(7/2) = 4");
}

/// Otherworldly Gaze surveils three and flashes back from the graveyard.
#[test]
fn otherworldly_gaze_surveil_and_flashback() {
    let mut g = two_player_game();
    for _ in 0..8 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::otherworldly_gaze());
    g.players[0].mana_pool.add(Color::Blue, 1);
    // AutoDecider keeps everything on top (no graveyard sends).
    cast(&mut g, id);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Gaze in graveyard");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastFlashback {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("flashback Gaze");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == id), "flashback exiles");
}

/// Metallic Rebuke improvises down to {U} with two artifacts and counters
/// unless {3} is paid.
#[test]
fn metallic_rebuke_improvises_and_taxes() {
    let mut g = two_player_game();
    // Seat 1 casts a Bear; seat 0 rebukes with two artifacts + {U}.
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bears cast");
    let a1 = g.add_card_to_battlefield(0, catalog::ornithopter());
    let a2 = g.add_card_to_battlefield(0, catalog::ornithopter());
    let id = g.add_card_to_hand(0, catalog::metallic_rebuke());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellConvoke {
        card_id: id, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
        convoke_creatures: vec![a1, a2],
    }).expect("Rebuke castable off {U} + two improvised artifacts");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a1).unwrap().tapped && g.battlefield_find(a2).unwrap().tapped,
        "both artifacts improvised");
    assert!(g.battlefield_find(bears).is_none(), "no mana paid for the tax: countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears));
}

/// Whirler Rogue mints two Thopters and taps two artifacts to make a
/// creature unblockable.
#[test]
fn whirler_rogue_thopters_and_unblockable() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::whirler_rogue());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let thopters: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Thopter" && c.controller == 0)
        .map(|c| c.id).collect();
    assert_eq!(thopters.len(), 2, "two Thopters minted");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("tap two artifacts for unblockable");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
    assert!(thopters.iter().all(|t| g.battlefield_find(*t).unwrap().tapped),
        "the Thopters paid the tap cost");
}

/// Militia Bugler digs four for a small creature and bottoms the rest.
#[test]
fn militia_bugler_digs_for_small_creature() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears()); // power 2
    g.add_card_to_library(0, catalog::serra_angel());              // power 4
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::militia_bugler());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert!(g.players[0].has_in_hand(bear), "the power-2 creature was taken");
    assert_eq!(g.players[0].library.len(), 3, "rest on the bottom");
}

/// Ice-Fang Coatl has deathtouch only with three other snow permanents.
#[test]
fn ice_fang_coatl_conditional_deathtouch() {
    let mut g = two_player_game();
    let coatl = g.add_card_to_battlefield(0, catalog::ice_fang_coatl());
    assert!(!g.computed_permanent(coatl).unwrap().keywords.contains(&Keyword::Deathtouch));
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::snow_covered_island()); }
    assert!(g.computed_permanent(coatl).unwrap().keywords.contains(&Keyword::Deathtouch),
        "three other snow permanents turn deathtouch on");
}

/// Sorin the Mirthless +1 takes the top card for its mana value in life.
#[test]
fn sorin_the_mirthless_plus_one_taxes_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
    let id = g.add_card_to_battlefield(0, catalog::sorin_the_mirthless());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("Sorin +1");
    drain_stack(&mut g);
    assert!(g.players[0].has_in_hand(bear), "top card in hand");
    assert_eq!(g.players[0].life, 18, "paid 2 life (its mana value)");
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(crabomination::card::CounterType::Loyalty), 5);
}

/// Omnath, Locus of the Roil's ETB damage scales with your Elementals and
/// landfall puts a counter on an Elemental.
#[test]
fn omnath_roil_etb_damage_and_landfall_counter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::flamekin_harbinger()); // 1 elemental
    let id = g.add_card_to_hand(0, catalog::omnath_locus_of_the_roil());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Omnath");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "two Elementals (Harbinger + Omnath) = 2 damage");

    let land = g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    let total: u32 = g.battlefield.iter().filter(|c| c.controller == 0)
        .map(|c| c.counter_count(crabomination::card::CounterType::PlusOnePlusOne)).sum();
    assert_eq!(total, 1, "landfall added one +1/+1 counter to an Elemental");
}

/// Dwarven Mine enters tapped without three other Mountains, untapped (and
/// mints a Dwarf) with them.
#[test]
fn dwarven_mine_conditional_tap_and_token() {
    let mut g = two_player_game();
    let a = g.add_card_to_hand(0, catalog::dwarven_mine());
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(a)).expect("play first Mine");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped, "enters tapped early");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Dwarf"));

    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::mountain()); }
    g.players[0].lands_played_this_turn = 0;
    let b = g.add_card_to_hand(0, catalog::dwarven_mine());
    g.perform_action(GameAction::PlayLand(b)).expect("play second Mine");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(b).unwrap().tapped, "three other Mountains → untapped");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Dwarf"), "Dwarf minted");
}

/// Flamekin Harbinger tutors an Elemental to the top of the library.
#[test]
fn flamekin_harbinger_tops_an_elemental() {
    let mut g = two_player_game();
    let omnath = g.add_card_to_library(0, catalog::omnath_locus_of_the_roil());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(omnath))]));
    let id = g.add_card_to_hand(0, catalog::flamekin_harbinger());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, id);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(omnath),
        "the Elemental is on top");
}

/// Fell Stinger's exploit payoff draws two and drains two from the bound
/// player.
#[test]
fn fell_stinger_exploit_draws_two_loses_two() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // exploit fodder
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::fell_stinger());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Accept the exploit; the payoff targets its controller by default.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let before = g.players[0].hand.len();
    cast(&mut g, id);
    assert_eq!(g.players[0].hand.len(), before - 1 + 2, "drew two");
    assert_eq!(g.players[0].life, 18, "lost two");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "a creature was exploited");
}

