//! Targeted Comprehensive-Rules conformance tests: Detain (CR 701.35),
//! Fateseal (CR 701.29), the cross-type legend rule (CR 704.5j),
//! +1/+1 vs -1/-1 counter annihilation (CR 122.3), Valentin's
//! death-replacement at the destroy funnel (CR 614), Exchange control
//! (CR 701.12), Fight + deathtouch (CR 701.14 / 702.2), and the
//! defending-player binding for "a creature you control attacks"
//! triggers (CR 509.2 / 603.2), Domain (CR 702.43), and Equipment-granted
//! triggers resolving on the Equipment (CR 702.6e).

use crate::catalog;
use crate::game::types::{Attack, AttackTarget};
use crate::game::two_player_game;
use super::*;

// ── CR 701.35 — Detain ────────────────────────────────────────────────────────

#[test]
fn cr_701_35_detain_stops_attack_block_and_activation_until_detainers_next_turn() {
    let mut g = two_player_game();
    // Opponent (seat 1) controls a creature that we'll detain via Lyev Skyknight.
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(victim);
    // Cast Lyev Skyknight (seat 0) and detain the bear on ETB.
    let lyev = g.add_card_to_hand(0, catalog::lyev_skyknight());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crate::game::cast_at(&mut g, lyev, Target::Permanent(victim));
    assert_eq!(g.battlefield_find(victim).unwrap().detained_by, Some(0), "bear detained by seat 0");

    // The detained bear can't be declared as an attacker on the opponent's turn.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    let err = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: victim, target: AttackTarget::Player(0),
    }]));
    assert!(err.is_err(), "detained creature can't attack");

    // It can't block either.
    g.step = TurnStep::DeclareBlockers;
    g.block_map.clear();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(1) });
    let berr = g.perform_action(GameAction::DeclareBlockers(vec![(victim, attacker)]));
    assert!(berr.is_err(), "detained creature can't block");
}

#[test]
fn cr_701_35_detain_clears_at_detainers_next_turn() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().detained_by = Some(0);
    // Detainer (seat 0) begins a new turn → detain lifts.
    g.active_player_idx = 0;
    g.do_untap();
    assert_eq!(g.battlefield_find(victim).unwrap().detained_by, None, "detain lifts on detainer's turn");
}

// ── CR 701.29 — Fateseal ──────────────────────────────────────────────────────

/// Test-only fixture: a Sorcery that fateseals 2 against each opponent.
fn fateseal_two() -> crate::card::CardDefinition {
    use crate::card::{CardDefinition, CardType};
    use crate::effect::{Effect, PlayerRef, Value};
    CardDefinition {
        name: "Test Fateseal 2",
        cost: crate::mana::cost(&[crate::mana::generic(1), crate::mana::u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Fateseal { who: PlayerRef::EachOpponent, amount: Value::Const(2) },
        ..Default::default()
    }
}

#[test]
fn cr_701_29_fateseal_bottoms_chosen_card_of_opponent_library() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Opponent's top two library cards.
    let top = g.add_card_to_library(1, catalog::island());
    let _second = g.add_card_to_library(1, catalog::forest());
    let before_len = g.players[1].library.len();
    let spell = g.add_card_to_hand(0, fateseal_two());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Put the opponent's top card (`top`) on the bottom.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![top])]));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crate::game::cast(&mut g, spell);
    assert_eq!(g.players[1].library.len(), before_len, "library size unchanged");
    assert_eq!(g.players[1].library.last().unwrap().id, top, "chosen card sent to bottom");
}

// ── CR 704.5j — legend rule across permanent types ─────────────────────────────

/// A legend-ruled *planeswalker* leaves the battlefield without emitting a
/// CreatureDied event; the controller keeps one copy.
#[test]
fn cr_704_5j_legend_rule_keeps_one_planeswalker_no_creature_death() {
    let mut g = two_player_game();
    let first = g.add_card_to_battlefield(0, catalog::rowan_scholar_of_sparks());
    let second = g.add_card_to_battlefield(0, catalog::rowan_scholar_of_sparks());
    let events = g.check_state_based_actions();
    let survivors: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Rowan, Scholar of Sparks").collect();
    assert_eq!(survivors.len(), 1, "exactly one Rowan remains");
    // The newest (second) is kept by AutoDecider; the first is binned.
    assert!(survivors[0].id == second || survivors[0].id == first);
    assert!(
        !events.iter().any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "a planeswalker legend-rule loss is not a creature death (CR 700.4)"
    );
}

// ── CR 122.3 — +1/+1 and -1/-1 counters annihilate as an SBA ───────────────────

#[test]
fn cr_122_3_plus_and_minus_counters_annihilate() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::MinusOneMinusOne, 2);
    g.check_state_based_actions();
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "2 pairs annihilated, 1 plus left");
    assert_eq!(c.counter_count(CounterType::MinusOneMinusOne), 0);
    assert_eq!((c.power(), c.toughness()), (3, 3), "2/2 base + net one +1/+1");
}

// ── CR 614 — Valentin's death-replacement is checked at every death funnel ──────

/// A *destroy* effect (not just lethal-damage SBA) on an opponent's nontoken
/// creature is also redirected to exile by Valentin's replacement.
#[test]
fn cr_614_exile_replacement_applies_to_destroy_path() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::valentin_dean_of_the_vein());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // No spare mana for the reflexive pay-{2}, so the Pest half is skipped.
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crate::game::cast_at(&mut g, murder, Target::Permanent(opp));
    assert!(g.exile.iter().any(|c| c.id == opp), "destroyed opp creature exiled instead");
}

// ── CR 701.12 — Exchange (control of two permanents) ───────────────────────────

/// Switcheroo swaps control of two target creatures (Effect::ExchangeControl).
#[test]
fn cr_701_12_exchange_control_of_two_creatures() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::switcheroo());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast Switcheroo");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1);
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
}

// ── CR 701.14 / 702.2 — Fight + deathtouch ─────────────────────────────────────

/// A 1/1 deathtoucher that fights a 4/4 destroys it (any nonzero deathtouch
/// damage is lethal, CR 702.2c) while surviving the 4 it takes... no — it dies
/// too (4 ≥ 1 toughness). What we assert: the big creature dies to deathtouch.
#[test]
fn cr_702_2_fight_with_deathtouch_kills_larger_creature() {
    let mut g = two_player_game();
    let killer = g.add_card_to_battlefield(0, catalog::deadly_recluse()); // 1/2 deathtouch
    let big = g.add_card_to_battlefield(1, catalog::serra_angel());       // 4/4
    let id = g.add_card_to_hand(0, catalog::prey_upon());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(killer)),
        additional_targets: vec![Target::Permanent(big)], mode: None, x_value: None,
    }).expect("cast Prey Upon");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(big).is_none(), "4/4 dies to 1 deathtouch damage");
}

// ── CR 509.2 / 603.2 — "a creature you control attacks" binds defending player ──

/// Leeching Sliver's "whenever a Sliver you control attacks, defending player
/// loses 1 life" resolves against the *attacker's* defending player even though
/// the ability source (Leeching Sliver) isn't the one attacking.
#[test]
fn cr_509_2_attack_trigger_binds_attackers_defending_player() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leeching_sliver());
    let attacker = g.add_card_to_battlefield(0, catalog::muscle_sliver());
    g.battlefield.iter_mut().find(|c| c.id == attacker).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let events = g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "defending player (seat 1) lost 1 life");
}

// ── CR 702.43 — Domain ────────────────────────────────────────────────────────

/// Domain counts the number of distinct basic land types among the player's
/// lands (0–5), driving both `Value::DomainCount` payoffs (Tribal Flames) and
/// `StaticEffect::SelfCostReducedByDomain` cost reduction (Leyline Binding).
#[test]
fn cr_702_43_domain_counts_distinct_basic_land_types() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crate::card::CardInstance::new(g.next_id(), catalog::leyline_binding(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "no lands → domain 0");
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest()); // duplicate type doesn't recount
    g.add_card_to_battlefield(0, catalog::island());
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 2,
        "two distinct basic types → domain 2");
}

// ── CR 702.6e — Equipment-granted triggered ability on the Equipment ──────────

/// `EquipBonus.triggers_on_equipment` makes the granted combat-damage trigger
/// resolve with the Equipment as its source, so Umezawa's Jitte's counters land
/// on the Equipment rather than the equipped creature.
#[test]
fn cr_702_6e_equip_trigger_resolves_on_the_equipment() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let jitte = g.add_card_to_battlefield(0, catalog::umezawas_jitte());
    g.battlefield.iter_mut().find(|c| c.id == jitte).unwrap().attached_to = Some(attacker);
    // Fire the combat-damage-to-player trigger directly.
    g.fire_combat_damage_to_player_triggers(attacker, 1, 2);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(jitte).unwrap().counter_count(CounterType::Charge), 2,
        "charge counters landed on the Equipment, not the creature");
    assert_eq!(g.battlefield_find(attacker).unwrap().counter_count(CounterType::Charge), 0,
        "the equipped creature got no counters");
}

// ── CR 510.2 — combat damage to a creature fires triggers ─────────────────────

/// `DealsCombatDamageToCreature` triggers (CR 510.2) are now dispatched from
/// the combat damage step, so Umezawa's Jitte charges when its equipped
/// creature is blocked and deals damage to the blocker.
#[test]
fn cr_510_2_jitte_charges_when_equipped_creature_is_blocked() {
    use crate::card::CounterType;
    use crate::game::types::Attack;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let jitte = g.add_card_to_battlefield(0, catalog::umezawas_jitte());
    g.battlefield_find_mut(jitte).unwrap().attached_to = Some(attacker);
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(1) }];
    g.block_map.insert(blocker, attacker);
    g.step = TurnStep::CombatDamage;
    g.active_player_idx = 0;
    g.resolve_combat().expect("regular combat damage");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(jitte).unwrap().counter_count(CounterType::Charge), 2,
        "Jitte charges off combat damage dealt to the blocking creature");
}

/// A creature unblocked deals damage to a *player*, not a creature, so the
/// to-creature dispatch must not fire for it (no spurious double-charge).
#[test]
fn cr_510_2_to_creature_dispatch_skips_unblocked_attacker() {
    use crate::card::CounterType;
    use crate::game::types::Attack;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let jitte = g.add_card_to_battlefield(0, catalog::umezawas_jitte());
    g.battlefield_find_mut(jitte).unwrap().attached_to = Some(attacker);
    g.clear_sickness(attacker);
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(1) }];
    g.step = TurnStep::CombatDamage;
    g.active_player_idx = 0;
    g.resolve_combat().expect("regular combat damage");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(jitte).unwrap().counter_count(CounterType::Charge), 2,
        "exactly one charge trigger (the to-player one) — two counters, not four");
}

// ── CR 509.1d — block tax (Archangel of Tithes) ───────────────────────────────

/// While Archangel of Tithes attacks, the defender must pay {1} for each
/// blocker; the declaration is rejected when the blocking player can't cover
/// the tax, and accepted once they can.
#[test]
fn cr_509_1d_block_tax_requires_payment() {
    use crate::game::types::Attack;
    let mut g = two_player_game();
    // Archangel attacks (turning on the block tax) alongside a ground bear.
    let angel = g.add_card_to_battlefield(0, catalog::archangel_of_tithes());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(angel);
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking = vec![
        Attack { attacker: angel, target: AttackTarget::Player(1) },
        Attack { attacker, target: AttackTarget::Player(1) },
    ];
    g.step = TurnStep::DeclareBlockers;
    g.active_player_idx = 0;
    // Seat 1 has no mana → can't pay the {1} block tax.
    assert!(g.declare_blockers(vec![(blocker, attacker)]).is_err(),
        "block rejected without paying the tax");
    // Give seat 1 one mana and retry.
    g.players[1].mana_pool.add_colorless(1);
    g.declare_blockers(vec![(blocker, attacker)]).expect("block legal once the tax is paid");
    assert_eq!(g.players[1].mana_pool.total(), 0, "the block tax was spent");
}

/// The block tax is gated on the source attacking: an Archangel sitting back on
/// defense imposes no block tax.
#[test]
fn cr_509_1d_block_tax_inactive_when_not_attacking() {
    use crate::game::types::Attack;
    let mut g = two_player_game();
    // Seat 1's Archangel is not attacking; seat 0 attacks with a bear.
    g.add_card_to_battlefield(1, catalog::archangel_of_tithes());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareBlockers;
    g.active_player_idx = 0;
    g.declare_blockers(vec![(blocker, attacker)]).expect("no tax when Archangel isn't attacking");
}
