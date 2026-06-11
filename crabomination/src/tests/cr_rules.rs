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

// ── Mass exilers fire leaves-graveyard bookkeeping ────────────────────────────

/// Rest in Peace's sweep counts as cards leaving each graveyard (the
/// Witherbloom leaves-graveyard payoffs must see mass exilers too).
#[test]
fn mass_graveyard_exile_fires_left_graveyard_bookkeeping() {
    use crate::effect::{Effect, PlayerRef};
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    for p in 0..2 {
        let id = g.next_id();
        g.players[p].graveyard.push(crate::card::CardInstance::new(
            id,
            catalog::lightning_bolt(),
            p,
        ));
    }
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(&Effect::ExilePlayerGraveyard { who: PlayerRef::EachPlayer }, &ctx)
        .unwrap();
    for p in 0..2 {
        assert_eq!(g.players[p].cards_left_graveyard_this_turn, 1, "P{p} tally");
    }
    assert_eq!(
        events.iter().filter(|e| matches!(e, GameEvent::CardLeftGraveyard { .. })).count(),
        2
    );
}

// ── CR 702.80a / 702.90e / 702.2c — keyworded NON-combat damage ───────────────

/// Test-only fixture: a creature with the given keywords.
fn kw_creature(
    name: &'static str,
    p: i32,
    t: i32,
    kws: &[crate::card::Keyword],
) -> crate::card::CardDefinition {
    use crate::card::{CardDefinition, CardType};
    CardDefinition {
        name,
        cost: crate::mana::cost(&[crate::mana::g()]),
        card_types: vec![CardType::Creature],
        power: p,
        toughness: t,
        keywords: kws.to_vec(),
        ..Default::default()
    }
}

/// Non-combat damage from a wither source lands as -1/-1 counters, not
/// marked damage (CR 702.80a; infect does the same to creatures, 702.90e).
#[test]
fn cr_702_80a_noncombat_wither_damage_is_minus_counters() {
    use crate::card::Keyword;
    use crate::effect::{Effect, Selector, Value};
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, kw_creature("Withertest", 2, 2, &[Keyword::Wither]));
    let tgt = g.add_card_to_battlefield(1, catalog::serra_angel());
    let ctx = EffectContext::for_ability(src, 0, Some(Target::Permanent(tgt)));
    g.resolve_effect(
        &Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) },
        &ctx,
    )
    .unwrap();
    let c = g.battlefield_find(tgt).unwrap();
    assert_eq!(c.counter_count(crate::card::CounterType::MinusOneMinusOne), 2);
    assert_eq!(c.damage, 0, "wither damage is not marked damage");
}

/// A nonzero non-combat ping from a deathtouch source destroys the damaged
/// creature at the next SBA check (CR 702.2c).
#[test]
fn cr_702_2c_noncombat_deathtouch_ping_destroys() {
    use crate::card::Keyword;
    use crate::effect::{Effect, Selector, Value};
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, kw_creature("Touchtest", 1, 1, &[Keyword::Deathtouch]));
    let tgt = g.add_card_to_battlefield(1, catalog::serra_angel());
    let ctx = EffectContext::for_ability(src, 0, Some(Target::Permanent(tgt)));
    g.resolve_effect(
        &Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(1) },
        &ctx,
    )
    .unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(tgt).is_none(), "4/4 dies to 1 deathtouch ping");
}

/// Fight halves carry their source: a lifelink fighter's controller gains
/// life equal to the damage it deals (CR 701.12b / 702.15).
#[test]
fn cr_701_12_fight_applies_lifelink_from_each_half() {
    use crate::card::Keyword;
    use crate::effect::{Effect, Selector};
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, kw_creature("Lifetest", 3, 5, &[Keyword::Lifelink]));
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    let life = g.players[0].life;
    let ctx = EffectContext::for_ability(mine, 0, Some(Target::Permanent(theirs)));
    g.resolve_effect(
        &Effect::Fight { attacker: Selector::This, defender: Selector::Target(0) },
        &ctx,
    )
    .unwrap();
    assert_eq!(g.players[0].life, life + 3, "lifelink fighter gains its damage dealt");
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

// ── CR 702.46 — Cipher ────────────────────────────────────────────────────────

/// A Cipher spell exiles encoded on a creature; when that creature deals combat
/// damage to a player, its controller casts a free copy.
#[test]
fn cr_702_46_cipher_encodes_then_recasts_on_combat_damage() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    use crate::game::types::Attack;
    let mut g = two_player_game();
    // Say "yes" to the encode prompt and the later free-copy prompt.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let slice = g.add_card_to_hand(0, catalog::shadow_slice());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    crate::game::cast_at(&mut g, slice, Target::Player(1));
    assert_eq!(g.players[1].life, 17, "Shadow Slice: 20 → 17");
    assert!(g.exile.iter().any(|c| c.id == slice && c.encoded_on == Some(bear)),
        "Shadow Slice exiled encoded on the bear");
    // Connect with the bear: 2 combat damage + a free Shadow Slice copy (−3).
    g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 12, "17 − 2 combat − 3 cipher copy");
}

// ── CR 614.9 — Damage redirection (Palisade Giant) ────────────────────────────

/// Noncombat damage to the controller or their other permanents lands on the
/// redirector; damage to the redirector itself applies normally.
#[test]
fn cr_614_9_palisade_giant_redirects_noncombat_damage() {
    let mut g = two_player_game();
    let giant = g.add_card_to_battlefield(0, catalog::palisade_giant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    // Bolt at the player: redirected to the giant.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
    crate::game::cast_at(&mut g, bolt, Target::Player(0));
    assert_eq!(g.players[0].life, 20, "player untouched");
    assert_eq!(g.battlefield_find(giant).unwrap().damage, 3, "giant soaked the bolt");
    // Bolt at the bear: also redirected.
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
    crate::game::cast_at(&mut g, bolt2, Target::Permanent(bear));
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "bear untouched");
    assert_eq!(g.battlefield_find(giant).unwrap().damage, 6, "giant soaked both");
}

/// Unblocked combat damage aimed at the redirector's controller is dealt to
/// the redirector instead.
#[test]
fn cr_614_9_palisade_giant_redirects_combat_damage_to_player() {
    let mut g = two_player_game();
    let giant = g.add_card_to_battlefield(0, catalog::palisade_giant());
    let attacker = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(0) }];
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "defender untouched");
    assert_eq!(g.battlefield_find(giant).unwrap().damage, 6, "giant took the hit");
}

// ── CR 702.103 — Jump-start ───────────────────────────────────────────────────

/// Jump-start casts from the graveyard for the card's own cost plus a
/// discard, and exiles after resolving.
#[test]
fn cr_702_103_jump_start_casts_from_graveyard_and_exiles() {
    let mut g = two_player_game();
    let spell = g.add_card_to_graveyard(0, catalog::radical_idea());
    let fodder = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFlashback {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("jump-start cast");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder), "discarded a card as cost");
    assert!(g.exile.iter().any(|c| c.id == spell), "exiled after resolving (702.103b)");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Island"), "drew the card");
}

/// Jump-start is rejected with an empty hand (the discard is unpayable).
#[test]
fn cr_702_103_jump_start_requires_a_discard() {
    let mut g = two_player_game();
    let spell = g.add_card_to_graveyard(0, catalog::radical_idea());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::CastFlashback {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no card to discard → can't jump-start");
}

// ── CR 728 — Ending the Turn ──────────────────────────────────────────────────

/// Sundial of the Infinite ends the turn: a spell still on the stack is
/// exiled (not resolved), combat state clears, and play skips to cleanup.
#[test]
fn cr_728_sundial_exiles_the_stack_and_skips_to_cleanup() {
    let mut g = two_player_game();
    let sundial = g.add_card_to_battlefield(0, catalog::sundial_of_the_infinite());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast bolt");
    g.perform_action(GameAction::ActivateAbility {
        card_id: sundial, ability_index: 0, target: None, x_value: None,
    }).expect("activate Sundial");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "bolt exiled off the stack (728.1a)");
    assert_eq!(g.players[1].life, 20, "bolt never resolved");
    assert_eq!(g.step, TurnStep::Cleanup, "turn skipped to cleanup (728.1d)");
}

/// Sundial's "activate only during your turn" gate rejects an off-turn use.
#[test]
fn cr_728_sundial_rejects_activation_on_opponents_turn() {
    let mut g = two_player_game();
    let sundial = g.add_card_to_battlefield(1, catalog::sundial_of_the_infinite());
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: sundial, ability_index: 0, target: None, x_value: None,
    }).is_err(), "only during your turn");
}

/// Day's Undoing wheels both players (hand + graveyard shuffled into
/// library, draw seven) and, on the caster's turn, ends the turn — the
/// sorcery itself is exiled with the stack (728.1a).
#[test]
fn cr_728_days_undoing_wheels_then_ends_the_turn() {
    let mut g = two_player_game();
    let du = g.add_card_to_hand(0, catalog::days_undoing());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(1, catalog::forest());
    }
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, du);
    assert_eq!(g.players[0].hand.len(), 7, "caster drew seven");
    assert_eq!(g.players[1].hand.len(), 7, "opponent drew seven");
    assert!(g.players[0].graveyard.is_empty(), "graveyard shuffled away");
    assert!(g.exile.iter().any(|c| c.id == du), "Day's Undoing exiled, not in graveyard");
    assert_eq!(g.step, TurnStep::Cleanup, "caster's turn ended");
}

// ── CR 615.7 — "a source of your choice" prevention ──────────────────────────

/// Burrenton Forge-Tender sacrificed in response to a red sweeper prevents
/// all the damage that spell would deal this turn.
#[test]
fn cr_615_7_forge_tender_blanks_a_red_spell() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let tender = g.add_card_to_battlefield(0, catalog::burrenton_forge_tender());
    g.clear_sickness(tender);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(mine)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast bolt at the bear");
    // In response: sacrifice the Forge-Tender choosing the bolt as source.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bolt])]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tender, ability_index: 0, target: None, x_value: None,
    }).expect("sac the Forge-Tender");
    drain_stack(&mut g);
    let bear = g.battlefield_find(mine).expect("bear survives");
    assert_eq!(bear.damage, 0, "all bolt damage prevented (615.7)");
    assert!(g.battlefield_find(tender).is_none(), "Forge-Tender sacrificed");
}

/// The chosen red creature deals no combat damage this turn; damage TO it
/// still applies.
#[test]
fn cr_615_7_forge_tender_prevents_a_creatures_combat_damage() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let tender = g.add_card_to_battlefield(1, catalog::burrenton_forge_tender());
    g.clear_sickness(tender);
    let raider = g.add_card_to_battlefield(0, catalog::ball_lightning());
    g.clear_sickness(raider);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![raider])]));
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tender, ability_index: 0, target: None, x_value: None,
    }).expect("sac the Forge-Tender");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.attacking = vec![Attack { attacker: raider, target: AttackTarget::Player(1) }];
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat");
    assert_eq!(g.players[1].life, 20, "prevented source deals no combat damage");
}

// ── CR 702.126 — Improvise ────────────────────────────────────────────────────

/// Kappa Cannoneer casts for {1}{U} by tapping four artifacts via Improvise,
/// and its own artifact entry grows it.
#[test]
fn cr_702_126_improvise_taps_artifacts_for_generic() {
    let mut g = two_player_game();
    let arts: Vec<_> = (0..4)
        .map(|_| g.add_card_to_battlefield(0, catalog::welding_jar()))
        .collect();
    let id = g.add_card_to_hand(0, catalog::kappa_cannoneer());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellConvoke {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        convoke_creatures: arts.clone(),
    }).expect("improvise cast for {1}{U} + four artifact taps");
    drain_stack(&mut g);
    for a in &arts {
        assert!(g.battlefield_find(*a).unwrap().tapped, "helper artifact tapped");
    }
    let k = g.battlefield_find(id).expect("resolved");
    assert_eq!(
        k.counter_count(crate::card::CounterType::PlusOnePlusOne),
        1,
        "its own entry triggered the counter"
    );
}

/// Improvise rejects tapping a creature that isn't an artifact.
#[test]
fn cr_702_126_improvise_rejects_nonartifact_helpers() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let id = g.add_card_to_hand(0, catalog::kappa_cannoneer());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    assert!(g.perform_action(GameAction::CastSpellConvoke {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        convoke_creatures: vec![bear],
    }).is_err(), "a creature can't improvise");
}

// ── CR 611.2c / 613.7c — PumpPT durations ─────────────────────────────────────

/// A `Duration::Permanent` pump (Wall of Roots's -0/-1) must survive the
/// Cleanup step's EOT-bonus wipe.
#[test]
fn cr_611_2c_permanent_pump_survives_cleanup() {
    use crate::effect::{Duration, Selector, Value};
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_roots());
    let eff = Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(0),
        toughness: Value::Const(-1),
        duration: Duration::Permanent,
    };
    let ctx = EffectContext::for_ability(wall, 0, None);
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.battlefield_find(wall).unwrap().toughness(), 4, "-0/-1 applied");
    for card in g.battlefield.iter_mut() {
        card.clear_end_of_turn_effects();
    }
    g.expire_end_of_turn_effects();
    assert_eq!(
        g.battlefield_find(wall).unwrap().toughness(),
        4,
        "permanent pump persists past cleanup"
    );
}

/// An `EndOfCombat` pump expires when the combat phase ends, not at Cleanup.
#[test]
fn cr_611_2c_end_of_combat_pump_expires_at_combat_end() {
    use crate::effect::{Duration, Selector, Value};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eff = Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(2),
        toughness: Value::Const(2),
        duration: Duration::EndOfCombat,
    };
    let ctx = EffectContext::for_ability(bear, 0, None);
    g.resolve_effect(&eff, &ctx).unwrap();
    let computed = g.computed_permanent(bear).expect("computed");
    assert_eq!((computed.power, computed.toughness), (4, 4), "pump active");
    g.expire_end_of_combat_effects();
    let computed = g.computed_permanent(bear).expect("computed");
    assert_eq!((computed.power, computed.toughness), (2, 2), "pump expired with combat");
}

/// A mid-duration pump aimed at a specific permanent ends when that permanent
/// leaves the battlefield and must not re-attach if it re-enters (CR 611.2c).
#[test]
fn cr_611_2c_specific_pump_does_not_follow_object_across_zones() {
    use crate::effect::{Duration, Selector, Value};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eff = Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(3),
        toughness: Value::Const(3),
        duration: Duration::UntilNextTurn,
    };
    let ctx = EffectContext::for_ability(bear, 0, None);
    g.resolve_effect(&eff, &ctx).unwrap();
    // Bounce and replay: the new object must be a plain 2/2.
    let mut events = Vec::new();
    g.move_card_to(bear, &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::You), &ctx, &mut events);
    let hand_pos = g.players[0].hand.iter().position(|c| c.id == bear).unwrap();
    let card = g.players[0].hand.remove(hand_pos);
    g.battlefield.push(card);
    let computed = g.computed_permanent(bear).expect("computed");
    assert_eq!((computed.power, computed.toughness), (2, 2), "pump ended on zone change");
}

// ── CR 510.1c — a blocked attacker remains blocked ────────────────────────────

/// Test-only fixture: a 3/3 double striker, optionally with trample.
fn double_striker(trample: bool) -> crate::card::CardDefinition {
    use crate::card::{CardDefinition, CardType, Keyword};
    let mut keywords = vec![Keyword::DoubleStrike];
    if trample {
        keywords.push(Keyword::Trample);
    }
    CardDefinition {
        name: "Test Double Striker",
        cost: crate::mana::cost(&[crate::mana::generic(2), crate::mana::r()]),
        card_types: vec![CardType::Creature],
        power: 3,
        toughness: 3,
        keywords,
        ..Default::default()
    }
}

/// A double striker whose blocker dies to first-strike damage stays blocked:
/// without trample, its regular-step damage hits nothing (CR 510.1c).
#[test]
fn cr_510_1c_blocked_attacker_stays_blocked_when_blocker_dies() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, double_striker(false));
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(blocker);
    let life_before = g.players[1].life;

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]))
        .unwrap();

    g.step = TurnStep::FirstStrikeDamage;
    g.resolve_first_strike_damage().unwrap();
    assert!(!g.battlefield.iter().any(|c| c.id == blocker), "blocker dies to FS damage");

    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert_eq!(
        g.players[1].life, life_before,
        "regular-step damage of a still-blocked attacker hits no one"
    );
}

/// With trample the same line assigns the regular-step damage to the
/// defending player (CR 702.19g).
#[test]
fn cr_702_19g_trample_attacker_hits_player_after_blocker_dies() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, double_striker(true));
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(blocker);
    let life_before = g.players[1].life;

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]))
        .unwrap();

    g.step = TurnStep::FirstStrikeDamage;
    g.resolve_first_strike_damage().unwrap();
    // FS step: 1 lethal to the 2/2 blocker, 1 tramples over.
    assert_eq!(g.players[1].life, life_before - 1, "trample overflow in the FS step");

    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert_eq!(
        g.players[1].life,
        life_before - 1 - 3,
        "all regular-step damage tramples through once the blocker is gone"
    );
}

// ── CR 601.2c — cast-time target filters enforced for every targeted effect ──

/// Detain was one of ~20 targeted effects whose filter wasn't surfaced by
/// `target_filter_for_slot`, letting a client submit any target (the caster's
/// own land). The filter must reject illegal targets at cast time.
#[test]
fn cr_601_2c_detain_filter_rejects_own_land() {
    use crate::card::{CardDefinition, CardType, SelectionRequirement};
    use crate::effect::{Effect, Selector};
    fn detain_spell() -> CardDefinition {
        CardDefinition {
            name: "Test Detain",
            cost: crate::mana::cost(&[crate::mana::u()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Detain {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Permanent
                        .and(SelectionRequirement::Nonland)
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
            ..Default::default()
        }
    }
    let mut g = two_player_game();
    let own_land = g.add_card_to_battlefield(0, catalog::island());
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, detain_spell());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(own_land)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "own land fails the detain filter"
    );
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_creature)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("opponent's creature is a legal detain target");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(opp_creature).unwrap().detained_by, Some(0));
}

/// CR 608.2b — a triggered ability whose stored sole target is illegal at
/// resolution fizzles; it must not re-aim at a fresh target.
#[test]
fn cr_608_2b_trigger_with_illegal_target_fizzles() {
    use crate::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    let bear_a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear_b = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let source = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Push a trigger aimed at bear_a, then remove bear_a before resolution.
    g.stack.push(crate::game::types::StackItem::Trigger {
        source,
        controller: 0,
        effect: Box::new(Effect::DealDamage {
            to: Selector::TargetFiltered {
                slot: 0,
                filter: crate::card::SelectionRequirement::Creature,
            },
            amount: Value::Const(2),
        }),
        target: Some(Target::Permanent(bear_a)),
        mode: None,
        x_value: 0,
        converged_value: 0,
        trigger_source: None,
        mana_spent: 0,
        event_amount: 0,
        intervening_if: None,
    });
    let mut events = Vec::new();
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.move_card_to(bear_a, &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::Seat(1)), &ctx, &mut events);
    g.resolve_top_of_stack().unwrap();
    let b = g.battlefield_find(bear_b).expect("untouched");
    assert_eq!(b.damage, 0, "fizzled trigger must not re-aim at another creature");
}

// ── Audit P1 batch: Rule of Law scope, steal sickness, detain loyalty,
//    Blood cost, Soulshift scope ──────────────────────────────────────────────

/// Rule of Law locks per game turn, not per the player's own untap-scoped
/// counter: a stale count from the player's previous turn must not lock them
/// out on an opponent's turn.
#[test]
fn cr_611_2_rule_of_law_does_not_lock_on_stale_counter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rule_of_law());
    // Simulate "P1 cast a spell on their own previous turn": stale per-untap
    // counter is 1, but no spell has been cast this game turn.
    g.players[1].spells_cast_this_turn = 1;
    g.players[1].spells_cast_this_game_turn = 0;
    let id = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("first spell this game turn is legal under Rule of Law");
    drain_stack(&mut g);
    // A second spell the same turn is locked.
    let id2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: id2, target: Some(Target::Player(0)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "second spell this turn is locked by Rule of Law"
    );
}

/// CR 302.6 — a stolen creature is summoning-sick under its new controller.
#[test]
fn cr_302_6_gain_control_sets_summoning_sickness() {
    use crate::effect::{Duration, Effect, Selector};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let eff = Effect::GainControl {
        what: Selector::TargetFiltered {
            slot: 0,
            filter: crate::card::SelectionRequirement::Creature,
        },
        to: None,
        duration: Duration::EndOfTurn,
    };
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.controller, 0, "control stolen");
    assert!(c.summoning_sick, "stolen creature is summoning-sick (no haste)");
}

/// CR 701.35 — a detained planeswalker can't activate loyalty abilities.
#[test]
fn cr_701_35_detained_planeswalker_cannot_activate_loyalty() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::teferi_time_raveler());
    g.battlefield_find_mut(pw).unwrap().detained_by = Some(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(
        g.activate_loyalty_ability(pw, 0, None, None).is_err(),
        "detained planeswalker's loyalty abilities are locked"
    );
}

/// CR 602.2b — the Blood token's discard is a cost: no hand card, no
/// activation (and no draw).
#[test]
fn cr_602_2b_blood_token_discard_is_a_cost() {
    use crate::game::effects::blood_token;
    let mut g = two_player_game();
    let blood = g.add_token_to_battlefield(0, &blood_token());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].hand.clear();
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: blood, ability_index: 0, target: None, x_value: None,
        })
        .is_err(),
        "empty hand can't pay the Blood discard cost"
    );
}

/// CR 702.47a — Soulshift returns a Spirit from YOUR graveyard, never an
/// opponent's.
#[test]
fn cr_702_47a_soulshift_only_fetches_own_graveyard() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::hundred_talon_kami());
    // Only the OPPONENT has a Spirit in their graveyard.
    let opp_spirit = g.add_card_to_graveyard(1, catalog::hundred_talon_kami());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let events = g.remove_to_graveyard_with_triggers(kami);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == opp_spirit),
        "opponent's Spirit stays in their graveyard"
    );
    assert!(
        !g.players[0].hand.iter().any(|c| c.id == opp_spirit),
        "soulshift must not steal an opponent's Spirit"
    );
}

/// CR 509.1a — an animated land (layer-4 Creature) can block; legality reads
/// the computed view, not printed types.
#[test]
fn cr_509_1a_animated_land_can_block() {
    use crate::effect::{Duration, Effect, Selector, Value};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let land = g.add_card_to_battlefield(0, catalog::forest());
    // Animate the land into a 3/3 creature for the turn.
    let eff = Effect::BecomeCreature {
        what: Selector::This,
        power: Value::Const(3),
        toughness: Value::Const(3),
        creature_types: vec![],
        keywords: vec![],
        duration: Duration::EndOfTurn,
    };
    let ctx = EffectContext::for_ability(land, 0, None);
    g.resolve_effect(&eff, &ctx).unwrap();

    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(land, attacker)]))
        .expect("animated land is a legal blocker");
    assert_eq!(g.block_map.get(&land), Some(&attacker));
}

// ── CR 510.1c/d — marked damage + full assignment ─────────────────────────────

/// A double-strike trampler only needs the blocker's REMAINING toughness as
/// lethal in the regular step — the rest tramples over (CR 510.1c).
#[test]
fn cr_510_1c_marked_damage_counts_toward_lethal() {
    use crate::card::{CardDefinition, CardType, Keyword};
    fn ds_trampler() -> CardDefinition {
        CardDefinition {
            name: "Test DS Trampler",
            cost: crate::mana::cost(&[crate::mana::generic(3), crate::mana::r()]),
            card_types: vec![CardType::Creature],
            power: 4,
            toughness: 4,
            keywords: vec![Keyword::DoubleStrike, Keyword::Trample],
            ..Default::default()
        }
    }
    fn wall_3_6() -> CardDefinition {
        CardDefinition {
            name: "Test Wall 3/6",
            cost: crate::mana::cost(&[crate::mana::generic(2)]),
            card_types: vec![CardType::Creature],
            power: 0,
            toughness: 6,
            ..Default::default()
        }
    }
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, ds_trampler());
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, wall_3_6());
    g.clear_sickness(blocker);
    let life = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).unwrap();
    g.step = TurnStep::FirstStrikeDamage;
    g.resolve_first_strike_damage().unwrap();
    // FS step: 4 to the 6-toughness wall (no overflow yet).
    assert_eq!(g.players[1].life, life, "no trample-over in the FS step");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    // Regular step: 4 marked already, lethal = 2 → 2 tramples through.
    assert_eq!(g.players[1].life, life - 2, "marked damage counted toward lethal");
}

/// Without trample the attacker's full power is assigned to its blocker —
/// the excess doesn't vanish (CR 510.1d).
#[test]
fn cr_510_1d_excess_damage_assigned_to_blocker() {
    use crate::card::{CardDefinition, CardType, Keyword};
    fn indestructible_2_2() -> CardDefinition {
        CardDefinition {
            name: "Test Indestructible Bear",
            cost: crate::mana::cost(&[crate::mana::generic(2)]),
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Indestructible],
            ..Default::default()
        }
    }
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::ulamog_the_ceaseless_hunger());
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, indestructible_2_2());
    g.clear_sickness(blocker);
    for _ in 0..25 {
        g.add_card_to_library(1, catalog::forest());
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert_eq!(
        g.battlefield_find(blocker).unwrap().damage,
        10,
        "all ten damage assigned to the lone blocker"
    );
}

// ── CR 119.10 — lifegain events carry the applied amount ─────────────────────

/// A fully-suppressed gain emits no LifeGained event (no lifegain triggers).
#[test]
fn cr_119_10_suppressed_gain_emits_no_event() {
    use crate::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::platinum_emperion());
    let eff = Effect::GainLife { who: Selector::You, amount: Value::Const(5) };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&eff, &ctx).unwrap();
    assert!(
        !events.iter().any(|e| matches!(e, GameEvent::LifeGained { .. })),
        "no LifeGained for a gain that never happened"
    );
}

// ── CR 122.1b — RemoveAllCounters clears keyword counters too ────────────────

#[test]
fn cr_122_1b_remove_all_counters_clears_keyword_counters() {
    use crate::card::Keyword;
    use crate::effect::{Effect, Selector};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().keyword_counters.insert(Keyword::Flying, 1);
    g.battlefield_find_mut(bear)
        .unwrap()
        .add_counters(crate::card::CounterType::PlusOnePlusOne, 2);
    let eff = Effect::RemoveAllCounters { what: Selector::This };
    let ctx = EffectContext::for_ability(bear, 0, None);
    g.resolve_effect(&eff, &ctx).unwrap();
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.counters.is_empty(), "regular counters cleared");
    assert!(c.keyword_counters.is_empty(), "keyword counters cleared (CR 122.1b)");
}
