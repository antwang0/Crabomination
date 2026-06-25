//! Targeted Comprehensive-Rules conformance tests: Detain (CR 701.35),
//! Fateseal (CR 701.29), the cross-type legend rule (CR 704.5j),
//! +1/+1 vs -1/-1 counter annihilation (CR 122.3), Valentin's
//! death-replacement at the destroy funnel (CR 614), Exchange control
//! (CR 701.12), Fight + deathtouch (CR 701.14 / 702.2), and the
//! defending-player binding for "a creature you control attacks"
//! triggers (CR 509.2 / 603.2), Domain (CR 702.43), Equipment-granted
//! triggers resolving on the Equipment (CR 702.6e), Prototype (CR 702.160),
//! Ward—pay-life-equal-to-power (CR 702.21), once-each-turn triggers
//! (CR 603.3d), Defender + conditional attack override (CR 702.12),
//! delayed "when you cast your next spell" triggers (CR 603.7e), counters
//! ceasing to exist on a zone change (CR 122.2), Gift (CR 702.165),
//! Survival (CR 702.180), the fixed-threshold power block restriction
//! (CR 509.1b — Questing Beast), "lose half your life, rounded up"
//! (CR 120.6), "up to N target" spells accepting fewer targets
//! (CR 601.2c), scoped unpreventable combat damage (CR 615.12 — Questing
//! Beast vs. fog), the stun-counter untap replacement (CR 122.1c),
//! Cycling (CR 702.29), Cleave bracket-removal (CR 702.148), Training
//! (CR 702.149), Exploit payoffs (CR 702.110), and reflexive
//! sacrifice costs (CR 701.16 — `Effect::MaySacrifice`), token-created
//! triggers (CR 111.10 — `EventKind::TokenCreated`) including the
//! token-doubling interaction (CR 614.13), and attack-only "can't attack
//! unless you control N" restrictions (CR 508.1a).

use crate::catalog;
use crate::card::CounterType;
use crate::game::types::{Attack, AttackTarget};
use crate::mana::Color;
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

// ── CR 601.2h-style combat-declaration atomicity ──────────────────────────────

/// A rejected attack declaration leaves no partial state: the legal
/// attacker in the batch is not tapped when a later one is illegal.
#[test]
fn rejected_attack_batch_taps_nothing() {
    let mut g = two_player_game();
    let ok = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(ok);
    let sick = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // summoning-sick
    g.step = TurnStep::DeclareAttackers;
    let err = g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: ok, target: AttackTarget::Player(1) },
        Attack { attacker: sick, target: AttackTarget::Player(1) },
    ]));
    assert!(err.is_err(), "sick attacker rejects the batch");
    assert!(!g.battlefield_find(ok).unwrap().tapped, "legal attacker untouched");
    assert!(g.attacking.is_empty(), "no attacker committed");
}

/// The same blocker twice in one batch is rejected (block_map is a single
/// blocker→attacker mapping; a duplicate would un-block the first).
#[test]
fn duplicate_blocker_in_batch_rejected() {
    let mut g = two_player_game();
    let a1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let a2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a1);
    g.clear_sickness(a2);
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a1, target: AttackTarget::Player(1) },
        Attack { attacker: a2, target: AttackTarget::Player(1) },
    ]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    let err = g.perform_action(GameAction::DeclareBlockers(vec![(b, a1), (b, a2)]));
    assert!(err.is_err(), "one creature can't block two attackers");
    assert!(g.block_map.is_empty(), "nothing committed");
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

/// CR 702.6e — an Equipment's granted triggered ability with
/// `triggers_on_equipment == false` fires as though printed on the equipped
/// creature, even for a non-combat observer event. Tarrian's Soulcleaver
/// ("whenever another artifact or creature is put into a graveyard, put a
/// +1/+1 counter on equipped creature") grows the host when an unrelated
/// permanent dies.
#[test]
fn cr_702_6e_equip_observer_trigger_fires_off_creature() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cleaver = g.add_card_to_battlefield(0, catalog::tarrians_soulcleaver());
    g.battlefield_find_mut(cleaver).unwrap().attached_to = Some(host);
    // An unrelated creature dies → the observer trigger grows the host.
    let fodder = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(fodder).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(host).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "equip-granted observer trigger put a +1/+1 counter on the equipped creature");
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
        card_id: sundial, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Sundial");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "bolt exiled off the stack (728.1a)");
    assert_eq!(g.players[1].life, 20, "bolt never resolved");
    // CR 728.1d + 514.3 — the turn skips to cleanup, which grants no
    // priority and ends the turn: play resumes in the opponent's upkeep.
    assert_eq!(g.active_player_idx, 1, "turn ended (728.1d)");
    assert_eq!(g.step, TurnStep::Upkeep, "no cleanup priority (514.3)");
}

/// Sundial's "activate only during your turn" gate rejects an off-turn use.
#[test]
fn cr_728_sundial_rejects_activation_on_opponents_turn() {
    let mut g = two_player_game();
    let sundial = g.add_card_to_battlefield(1, catalog::sundial_of_the_infinite());
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: sundial, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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
    assert_eq!(g.active_player_idx, 1, "caster's turn ended (728.1d + 514.3)");
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
        card_id: tender, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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
        card_id: tender, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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
        additional_targets: Vec::new(),
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
            card_id: blood, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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

// ── CR 702.90 / 615.6 — per-source blocker strike-back ───────────────────────

/// Each blocker's strike-back is its own damage event: only the infect
/// blocker's share becomes -1/-1 counters; the vanilla blocker's share is
/// marked damage (CR 702.90e is per source, not per step total).
#[test]
fn cr_702_90_infect_blocker_share_is_counters_only() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, kw_creature("Brute", 4, 20, &[]));
    g.clear_sickness(atk);
    let infect = g.add_card_to_battlefield(1, kw_creature("Sting", 2, 5, &[Keyword::Infect]));
    let vanilla = g.add_card_to_battlefield(1, kw_creature("Bear", 3, 5, &[]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(infect, atk), (vanilla, atk)])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    let a = g.battlefield_find(atk).unwrap();
    assert_eq!(a.counter_count(crate::card::CounterType::MinusOneMinusOne), 2,
        "only the infect blocker's 2 power lands as counters");
    assert_eq!(a.damage, 3, "the vanilla blocker's 3 power is marked damage");
}

/// Source-scoped damage scaling (Torbran) applies per strike-back event,
/// not once to the summed total (CR 614.5).
#[test]
fn cr_702_90_strike_back_scaling_is_per_source() {
    use crate::card::{CardDefinition, CardType};
    let red_body = |name: &'static str, p: i32, t: i32| CardDefinition {
        name,
        cost: crate::mana::cost(&[crate::mana::r()]),
        card_types: vec![CardType::Creature],
        power: p,
        toughness: t,
        ..Default::default()
    };
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, kw_creature("Brute", 0, 20, &[]));
    g.clear_sickness(atk);
    g.add_card_to_battlefield(1, catalog::torbran_thane_of_red_fell());
    let b1 = g.add_card_to_battlefield(1, red_body("Ember A", 2, 4));
    let b2 = g.add_card_to_battlefield(1, red_body("Ember B", 3, 4));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, atk), (b2, atk)])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    // Torbran adds +2 to each red source's event: (2+2) + (3+2) = 9.
    assert_eq!(g.battlefield_find(atk).unwrap().damage, 9);
}

// ── CR 613.7 — coherent timestamps between statics and resolved effects ──────

/// A spell's layer-6 effect resolving after an anthem's source entered
/// applies later in timestamp order (CR 613.7a/b/d) — statics no longer
/// carry CardId-space stamps that dwarf the effect counter.
#[test]
fn cr_613_7_later_removal_beats_earlier_static_grant() {
    use crate::card::{CardDefinition, CardType, Keyword, SelectionRequirement, StaticAbility};
    use crate::effect::{Duration, Effect, Selector, StaticEffect};
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, CardDefinition {
        name: "Wind Totem",
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Flying,
            },
        }],
        ..Default::default()
    });
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying));
    let ctx = EffectContext::for_ability(bear, 0, None);
    g.resolve_effect(
        &Effect::LoseAllAbilities { what: Selector::This, duration: Duration::EndOfTurn },
        &ctx,
    )
    .unwrap();
    assert!(
        !g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying),
        "later RemoveAllAbilities applies after the earlier static grant"
    );
}

/// Attaching re-stamps the Equipment (CR 613.7e), so its keyword grant
/// applies after an older ability-removal effect.
#[test]
fn cr_613_7e_attach_restamps_equipment_grant() {
    use crate::card::{ArtifactSubtype, CardDefinition, CardType, EquipBonus, Keyword, Subtypes};
    use crate::effect::{Duration, Effect, Selector};
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wings = g.add_card_to_battlefield(0, CardDefinition {
        name: "Strap-On Wings",
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        ..Default::default()
    });
    // Ability removal resolves after both permanents entered…
    let ctx = EffectContext::for_ability(bear, 0, None);
    g.resolve_effect(
        &Effect::LoseAllAbilities { what: Selector::This, duration: Duration::EndOfTurn },
        &ctx,
    )
    .unwrap();
    // …then the Equipment attaches: the attach re-stamp orders its grant last.
    let atx = EffectContext::for_ability(wings, 0, Some(Target::Permanent(bear)));
    let events = g
        .resolve_effect(&Effect::Attach { what: Selector::This, to: Selector::Target(0) }, &atx)
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying),
        "post-attach grant beats the earlier removal (CR 613.7e)"
    );
}

// ── CR 514.3 / 514.3a — cleanup priority only when something happens ─────────

/// A quiet cleanup grants no priority: passing out of the end step lands in
/// the next player's upkeep with the turn-based actions all done.
#[test]
fn cr_514_3_quiet_cleanup_grants_no_priority() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().damage = 1;
    g.step = TurnStep::End;
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.active_player_idx, 1, "turn ended without a cleanup window");
    assert_eq!(g.step, TurnStep::Upkeep);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "damage wore off (514.2)");
}

/// A cleanup discard that fires a trigger grants priority in the cleanup
/// step, and another cleanup round runs after the stack empties (514.3a).
#[test]
fn cr_514_3a_discard_trigger_grants_cleanup_priority_then_repeats() {
    use crate::card::{CardDefinition, CardType, TriggeredAbility};
    use crate::effect::{Effect, EventKind, EventScope, EventSpec, Selector, Value};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, CardDefinition {
        name: "Discard Payoff",
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    });
    for _ in 0..8 {
        g.add_card_to_hand(0, catalog::island());
    }
    g.step = TurnStep::End;
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    // The discard-down trigger granted a cleanup priority window.
    assert_eq!(g.step, TurnStep::Cleanup, "priority granted in cleanup (514.3a)");
    assert_eq!(g.players[0].hand.len(), 7, "discarded down to maximum (514.1)");
    assert!(!g.stack.is_empty(), "discard trigger on the stack");
    // Resolve the trigger, then both players pass: a repeat cleanup runs
    // quietly and the turn ends.
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.players[0].life, 21, "trigger resolved");
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.active_player_idx, 1, "repeat cleanup ended the turn");
}

// ── Audit P3 — walker exhaustiveness guard ──────────────────────────────────

/// Every `Selector::TargetFiltered` slot reachable in a catalog card's spell
/// effect must be surfaced by `target_filter_for_slot_in_mode_kicked` for
/// some mode/kicker state — otherwise the filter is unenforced at cast time
/// (CR 601.2c; the audit-P0 "~20 unenforced variants" class). The walk uses
/// the effect's serde tree, so a new Effect variant holding a filtered
/// target can't dodge it by missing an `eff_find` arm.
#[test]
fn cr_601_2c_every_catalog_target_filter_is_surfaced() {
    use serde_json::Value as J;
    fn collect_slots(j: &J, out: &mut std::collections::BTreeSet<u8>) {
        match j {
            J::Object(m) => {
                if let Some(tf) = m.get("TargetFiltered")
                    && let Some(slot) = tf.get("slot").and_then(|s| s.as_u64())
                {
                    out.insert(slot as u8);
                }
                for v in m.values() {
                    collect_slots(v, out);
                }
            }
            J::Array(a) => {
                for v in a {
                    collect_slots(v, out);
                }
            }
            _ => {}
        }
    }
    let mut failures = Vec::new();
    for f in crate::catalog::all_known_factories() {
        let def = f();
        let j = serde_json::to_value(&def.effect).expect("effect serializes");
        let mut slots = std::collections::BTreeSet::new();
        collect_slots(&j, &mut slots);
        for slot in slots {
            let surfaced = [false, true].iter().any(|&kicked| {
                std::iter::once(None).chain((0..8).map(Some)).any(|mode| {
                    def.effect
                        .target_filter_for_slot_in_mode_kicked(slot, mode, kicked)
                        .is_some()
                })
            });
            if !surfaced {
                failures.push(format!("{} slot {}", def.name, slot));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "target filters not surfaced at cast time for: {failures:?}"
    );
}

/// CR 119.3c — life paid for a Phyrexian pip is a life-loss event:
/// "whenever you lose life" triggers fire on the paid cost.
#[test]
fn cr_119_3c_phyrexian_life_payment_fires_loss_trigger() {
    use crate::card::{CardDefinition, CardType, TriggeredAbility};
    use crate::effect::{Effect, EventKind, EventScope, EventSpec, Selector, Value};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, CardDefinition {
        name: "Loss Payoff",
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeLost, EventScope::YourControl),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    });
    g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_hand(0, CardDefinition {
        name: "Phyrexian Bolt",
        cost: crate::mana::cost(&[crate::mana::phyrexian(crate::mana::Color::Black)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Noop,
        ..Default::default()
    });
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // No black mana floated — the pip is paid with 2 life.
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast paying life");
    assert_eq!(g.players[0].life, 18, "paid 2 life for the pip");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "life-loss trigger drew a card");
}

/// CR 608.2b — an Aura spell whose enchant target dies in response is
/// countered on resolution: it never enters the battlefield.
#[test]
fn cr_608_2b_aura_fizzles_when_enchant_target_dies() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::pacifism());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Pacifism on the bear");
    // The bear dies in response.
    g.remove_to_graveyard_with_triggers(bear);
    drain_stack(&mut g);
    assert!(
        !g.battlefield.iter().any(|c| c.id == aura),
        "fizzled Aura never enters the battlefield"
    );
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == aura),
        "fizzled Aura is countered into its owner's graveyard"
    );
}

/// CR 608.2b — an Aura still resolves and attaches when its target is legal.
#[test]
fn cr_608_2b_aura_attaches_to_legal_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::pacifism());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Pacifism");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(aura).and_then(|c| c.attached_to),
        Some(bear),
        "Aura attached to its target"
    );
}

// ── CR 700.4 — exile-instead replacements mean the creature never dies ──────

/// Under Rest in Peace a destroyed creature is exiled instead of dying:
/// "whenever a creature dies" watchers don't fire and Persist doesn't return.
#[test]
fn cr_700_4_exiled_instead_creatures_do_not_die() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rest_in_peace());
    g.add_card_to_battlefield(0, catalog::blood_artist());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let (you, opp) = (g.players[0].life, g.players[1].life);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    cast_at(&mut g, bolt, Target::Permanent(victim));
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "exiled instead of the graveyard");
    assert_eq!(g.players[0].life, you, "Blood Artist saw no death — no gain");
    assert_eq!(g.players[1].life, opp, "…and no drain");
}

/// Persist can't return a creature whose death-placement went to exile.
#[test]
fn cr_700_4_persist_does_not_return_from_exile_redirect() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rest_in_peace());
    let persister = g.add_card_to_battlefield(1, catalog::kitchen_finks());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::Destroy { what: crate::card::Selector::EachPermanent(
        crate::card::SelectionRequirement::Creature) }, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == persister), "in exile");
    assert!(g.battlefield_find(persister).is_none(), "did not persist back");
}

// ── CR 704.5e — a spell copy off the stack ceases to exist ──────────────────

/// Memory Lapse on a spell COPY doesn't shuffle a phantom card into the
/// library — the copy ceases to exist once off the stack.
#[test]
fn cr_704_5e_countered_spell_copy_ceases_to_exist() {
    let mut g = two_player_game();
    // Opponent casts a Bolt; we Reverberate it, then Memory Lapse the copy.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("opponent bolt");
    let fork = g.add_card_to_hand(0, catalog::reverberate());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: fork, target: Some(Target::Permanent(bolt)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Reverberate the bolt");
    // Resolve Reverberate only — the copy is now on the stack.
    g.resolve_top_of_stack().expect("fork resolves");
    let copy_id = match g.stack.last() {
        Some(crate::game::StackItem::Spell { card, .. }) if card.is_token => card.id,
        other => panic!("expected the copy on top, got {other:?}"),
    };
    let lapse = g.add_card_to_hand(0, catalog::memory_lapse());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: lapse, target: Some(Target::Permanent(copy_id)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lapse the copy");
    let lib_before = g.players[1].library.len();
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert_eq!(g.players[1].library.len(), lib_before, "no phantom card in the library");
    assert!(!g.players[1].library.iter().any(|c| c.id == copy_id));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt) || g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "the real Bolt resolved/died normally");
}

// ── CR 702.61 — Split second ─────────────────────────────────────────────────

/// While a split-second spell is on the stack, no player may cast spells or
/// activate non-mana abilities; mana abilities stay legal (702.61a-b).
#[test]
fn cr_702_61_split_second_locks_casts_and_nonmana_abilities() {
    let mut g = two_player_game();
    let shock = g.add_card_to_hand(0, catalog::sudden_shock());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());

    g.perform_action(GameAction::CastSpell {
        card_id: shock, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sudden Shock");
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap_err();
    assert_eq!(err, GameError::SplitSecondLock, "no responses under split second");
    // Mana abilities are exempt (702.61b).
    g.battlefield_find_mut(mountain).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mountain, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tapping for mana stays legal");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "Sudden Shock resolved for 2");
    // Lock lifts once the spell leaves the stack.
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable again after resolution");
}

/// Triggered abilities still trigger and go on the stack under split second
/// (CR 702.61b), and a non-mana activated ability is rejected.
#[test]
fn cr_702_61_triggers_fire_but_activations_blocked() {
    let mut g = two_player_game();
    // Opponent has a cast trigger watcher and an activatable artifact.
    g.add_card_to_battlefield(1, catalog::thermo_alchemist());
    let edict = g.add_card_to_hand(0, catalog::sudden_edict());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    g.clear_sickness(stone);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpell {
        card_id: edict, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sudden Edict");
    // Mind Stone's draw ability ({1},{T},Sac: draw) is not a mana ability.
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add_colorless(1);
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).unwrap_err();
    assert_eq!(err, GameError::SplitSecondLock);
    drain_stack(&mut g);
    assert!(g.battlefield_find(stone).is_some(), "Mind Stone never sacrificed");
}

// ── CR 702.64 — Absorb ────────────────────────────────────────────────────────

/// Lymph Sliver grants all Slivers absorb 1: each damage event to a Sliver
/// is reduced by 1, per source per event.
#[test]
fn cr_702_64_absorb_prevents_n_per_damage_event() {
    use crate::effect::{Selector, Value};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lymph_sliver());
    let subject = g.add_card_to_battlefield(0, catalog::crystalline_sliver()); // 2/2
    // Hit the 2/2 Sliver for 2 → absorb 1 → 1 marked damage, survives.
    let ctx = crate::game::effects::EffectContext::for_spell(
        1, Some(Target::Permanent(subject)), 0, 0,
    );
    g.resolve_effect(
        &Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) },
        &ctx,
    ).unwrap();
    assert_eq!(
        g.battlefield_find(subject).map(|c| c.damage),
        Some(1),
        "absorb 1 soaks one of the two"
    );
    // A second event is absorbed separately: 1 damage → fully prevented.
    g.resolve_effect(
        &Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(1) },
        &ctx,
    ).unwrap();
    assert_eq!(g.battlefield_find(subject).map(|c| c.damage), Some(1), "1-damage event fully absorbed");
}

/// Absorb applies to combat damage and keeps a 1-power attacker from
/// denting the Sliver at all.
#[test]
fn cr_702_64_absorb_soaks_combat_damage() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lymph_sliver());
    let sliver = g.add_card_to_battlefield(0, catalog::crystalline_sliver()); // 2/2 absorb 1
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(bear);
    g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(0) }];
    g.block_map.insert(sliver, bear);
    g.step = TurnStep::CombatDamage;
    g.active_player_idx = 1;
    g.resolve_combat().unwrap();
    // Bear deals 2, absorb 1 → 1 marked (survives); the 2/2 strikes back
    // for 2 and the bear dies.
    assert_eq!(g.battlefield_find(sliver).map(|c| c.damage), Some(1),
        "2/2 with absorb survives the 2-power hit with 1 marked");
    assert!(g.battlefield_find(bear).is_none(), "bear takes the full strike-back");
}

// ── CR 704.5y — Role uniqueness SBA ───────────────────────────────────────────

/// Two Roles controlled by the same player on one permanent: the older one
/// goes to the graveyard (and, being a token, ceases to exist).
#[test]
fn cr_704_5y_same_controller_roles_keep_only_the_newest() {
    use crate::card::{EnchantmentSubtype, EquipBonus, TokenDefinition};
    use crate::effect::Selector;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let role = TokenDefinition {
        name: "Wicked".into(),
        card_types: vec![crate::card::CardType::Enchantment],
        subtypes: crate::card::Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Role],
            ..Default::default()
        },
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        ..Default::default()
    };
    let ctx = crate::game::effects::EffectContext::for_ability(bear, 0, None);
    let mint = Effect::CreateTokenAttachedTo { target: Selector::This, definition: role };
    g.resolve_effect(&mint, &ctx).unwrap();
    let first: Vec<CardId> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Wicked").map(|c| c.id).collect();
    assert_eq!(first.len(), 1);
    g.resolve_effect(&mint, &ctx).unwrap();
    g.check_state_based_actions();
    let roles: Vec<&crate::card::CardInstance> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Wicked").collect();
    assert_eq!(roles.len(), 1, "only one Role survives the SBA");
    assert_ne!(roles[0].id, first[0], "the newer Role is the survivor");
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.definition.name == "Wicked"),
        "the dead Role token ceases to exist"
    );
}

/// Roles controlled by different players coexist (CR 704.5y is
/// per-controller).
#[test]
fn cr_704_5y_different_controllers_roles_coexist() {
    use crate::card::{EnchantmentSubtype, TokenDefinition};
    use crate::effect::Selector;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let role = TokenDefinition {
        name: "Cursed".into(),
        card_types: vec![crate::card::CardType::Enchantment],
        subtypes: crate::card::Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Role],
            ..Default::default()
        },
        ..Default::default()
    };
    let mint = Effect::CreateTokenAttachedTo { target: Selector::This, definition: role };
    g.resolve_effect(&mint, &crate::game::effects::EffectContext::for_ability(bear, 0, None)).unwrap();
    g.resolve_effect(&mint, &crate::game::effects::EffectContext::for_ability(bear, 1, None)).unwrap();
    g.check_state_based_actions();
    let n = g.battlefield.iter().filter(|c| c.definition.name == "Cursed").count();
    assert_eq!(n, 2, "one Role per controller may stay");
}

/// Nylea, Keen-Eyed's nonland miss offers "put it into your graveyard";
/// accepting bins the reveal, declining leaves it on top.
#[test]
fn nylea_reveal_miss_may_go_to_graveyard() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let top = g.add_card_to_library(0, catalog::lightning_bolt()); // not a creature
    let nylea = g.add_card_to_battlefield(0, catalog::nylea_keen_eyed());
    g.clear_sickness(nylea);
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: nylea, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate the reveal");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "miss binned by choice");
}

// ── CR 702.47 — Splice onto Arcane ───────────────────────────────────────────

/// Splicing Glacial Ray onto an Arcane spell pays the splice cost, keeps the
/// spliced card in hand, and resolves the spliced text after the main effect.
#[test]
fn cr_702_47_splice_adds_text_and_keeps_the_card_in_hand() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let mists = g.add_card_to_hand(0, catalog::reach_through_mists());
    let ray = g.add_card_to_hand(0, catalog::glacial_ray());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpellSpliced {
        card_id: mists,
        splice_cards: vec![ray],
        target: None,
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    }).expect("cast Reach Through Mists splicing Glacial Ray");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2, "spliced Ray dealt 2");
    assert!(g.players[0].hand.iter().any(|c| c.id == ray),
        "spliced card stays in hand (CR 702.47a)");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == mists),
        "main spell resolved to the graveyard");
    // The graveyard copy lost the spliced text (CR 702.47e).
    assert!(g.players[0].graveyard.iter().find(|c| c.id == mists)
        .unwrap().spliced_effects.is_empty());
}

/// Splice is rejected when the main spell isn't of the required quality, and
/// the whole cast rolls back (no splice cost stranded).
#[test]
fn cr_702_47_splice_requires_the_quality() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt()); // not Arcane
    let ray = g.add_card_to_hand(0, catalog::glacial_ray());
    g.players[0].mana_pool.add(Color::Red, 3);
    assert!(g.perform_action(GameAction::CastSpellSpliced {
        card_id: bolt,
        splice_cards: vec![ray],
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    }).is_err(), "Bolt has no Arcane subtype");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3, "nothing paid");
}

/// You can't splice a card without enough mana for its splice cost.
#[test]
fn cr_702_47_splice_cost_is_additional() {
    let mut g = two_player_game();
    let mists = g.add_card_to_hand(0, catalog::reach_through_mists());
    let ray = g.add_card_to_hand(0, catalog::glacial_ray());
    g.players[0].mana_pool.add(Color::Blue, 1); // {U} only — splice {1}{R} unpayable
    assert!(g.perform_action(GameAction::CastSpellSpliced {
        card_id: mists,
        splice_cards: vec![ray],
        target: None,
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    }).is_err());
    assert!(g.players[0].hand.iter().any(|c| c.id == mists), "cast rolled back");
}

// ── CR 704.5k — the world rule ───────────────────────────────────────────────

/// A second World permanent sends the older one to the graveyard.
#[test]
fn cr_704_5k_world_rule_keeps_the_newest() {
    let mut g = two_player_game();
    let old = g.add_card_to_battlefield(0, catalog::concordant_crossroads());
    g.battlefield_find_mut(old).unwrap().battlefield_timestamp = 10;
    let new = g.add_card_to_battlefield(1, catalog::nether_void());
    g.battlefield_find_mut(new).unwrap().battlefield_timestamp = 20;
    g.check_state_based_actions();
    assert!(g.battlefield_find(old).is_none(), "older World binned");
    assert!(g.battlefield_find(new).is_some(), "newest World survives");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == old));
}

/// On a timestamp tie, ALL World permanents go (CR 704.5k second sentence).
#[test]
fn cr_704_5k_world_rule_tie_bins_all() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::concordant_crossroads());
    let b = g.add_card_to_battlefield(1, catalog::nether_void());
    g.battlefield_find_mut(a).unwrap().battlefield_timestamp = 7;
    g.battlefield_find_mut(b).unwrap().battlefield_timestamp = 7;
    g.check_state_based_actions();
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(),
        "simultaneous Worlds all die");
}

/// Nether Void taxes every spell {3} (countered when unpaid).
#[test]
fn nether_void_counters_unpaid_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::nether_void());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bolt");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "unpaid tax: countered");
    assert_eq!(g.players[1].life, 20, "no damage dealt");
}

/// Concordant Crossroads gives everything haste.
#[test]
fn concordant_crossroads_grants_haste_to_all() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::concordant_crossroads());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(g.computed_permanent(mine).unwrap().keywords.contains(&crate::card::Keyword::Haste));
    assert!(g.computed_permanent(theirs).unwrap().keywords.contains(&crate::card::Keyword::Haste));
}

// ── CR 121.5 / multi-pick reveals ────────────────────────────────────────────

/// CR 121.5 — a card put into hand by a look-and-pick (Impulse) is NOT
/// drawn: no CardDrawn event, and an opponent's "whenever an opponent
/// draws" trigger (Consecrated Sphinx) doesn't fire.
#[test]
fn cr_121_5_look_pick_to_hand_is_not_a_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::consecrated_sphinx());
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::impulse());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_hand = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Impulse");
    let events = drain_stack(&mut g);
    assert!(!events.iter().any(|e| matches!(e, GameEvent::CardDrawn { player: 0, .. })),
        "the Impulse pick is put into hand, not drawn");
    assert_eq!(g.players[1].hand.len(), p1_hand, "Sphinx never fired");
}

/// A `wants_ui` Dig Through Time caster gets a real two-card `ChooseCards`
/// pick over the top seven; the chosen pair lands in hand, the rest bottom.
#[test]
fn multi_pick_dig_through_time_chooses_two() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let mut lib = Vec::new();
    for _ in 0..7 { lib.push(g.add_card_to_library(0, catalog::island())); }
    let id = g.add_card_to_hand(0, catalog::dig_through_time());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dig Through Time");
    g.perform_action(GameAction::PassPriority).expect("active passes");
    g.perform_action(GameAction::PassPriority).expect("non-active passes → resolve");

    let pd = g.pending_decision.as_ref().expect("the pick is pending");
    match &pd.decision {
        crate::decision::Decision::ChooseCards { candidates, min, max, .. } => {
            assert_eq!(candidates.len(), 7, "all seven revealed");
            assert_eq!((*min, *max), (2, 2), "exactly two picks");
        }
        other => panic!("expected ChooseCards, got {other:?}"),
    }
    // Pick the 3rd and 6th revealed cards (not the auto top-two).
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Cards(
        vec![lib[2], lib[5]],
    ))).expect("submit the pick");
    assert!(g.players[0].has_in_hand(lib[2]) && g.players[0].has_in_hand(lib[5]),
        "the chosen pair is in hand");
    assert_eq!(g.players[0].library.len(), 5, "the rest stay in the library");
    assert!(!g.players[0].has_in_hand(lib[0]), "the auto top card was not taken");
}

/// Atraxa's "up to one card of each card type" multi-pick validates the
/// answer: two picks sharing their only card type keep just the first.
#[test]
fn atraxa_take_one_per_type_validates_distinct_types() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    // Top of library (revealed set): two instants + a land.
    let bolt_a = g.add_card_to_library(0, catalog::lightning_bolt());
    let bolt_b = g.add_card_to_library(0, catalog::lightning_bolt());
    let isle = g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::atraxa_grand_unifier());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Atraxa");
    drain_stack(&mut g);

    let pd = g.pending_decision.as_ref().expect("ETB pick pending");
    assert!(matches!(pd.decision, crate::decision::Decision::ChooseCards { .. }));
    // Ask for both Bolts and the Island: the second Bolt is dropped
    // (Instant already covered), the Island keeps its Land slot.
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Cards(
        vec![bolt_a, bolt_b, isle],
    ))).expect("submit the pick");
    assert!(g.players[0].has_in_hand(bolt_a), "first instant taken");
    assert!(g.players[0].has_in_hand(isle), "land taken");
    assert!(!g.players[0].has_in_hand(bolt_b), "duplicate-type pick dropped");
    assert!(g.players[0].library.iter().any(|c| c.id == bolt_b), "it went to the bottom");
}

// ── CR 702.85 — Heroic ────────────────────────────────────────────────────────

/// Hero of the Pride's Heroic fires when a spell you cast targets it: every
/// creature you control gets +1/+0 (seen on a non-targeted teammate).
#[test]
fn cr_702_85_heroic_pumps_team_when_targeted() {
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::hero_of_the_pride());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2, not targeted
    let spell = g.add_card_to_hand(0, catalog::infuriate());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(hero)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Infuriate targeting the hero");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 3, "Heroic gave the team +1/+0");
}

/// Heroic does NOT fire when the spell targets a different creature.
#[test]
fn cr_702_85_heroic_silent_when_not_targeted() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hero_of_the_pride());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::infuriate());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), // targets the bear, not the hero
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Infuriate on the bear");
    drain_stack(&mut g);
    // Bear got only Infuriate's +3/+2 (5/4), not an extra heroic +1/+0.
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 5, "no heroic pump");
}

/// Phalanx Leader's Heroic puts a +1/+1 counter on each creature you control.
#[test]
fn cr_702_85_phalanx_leader_counters_team() {
    let mut g = two_player_game();
    let leader = g.add_card_to_battlefield(0, catalog::phalanx_leader());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::infuriate());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(leader)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Infuriate targeting Phalanx Leader");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "teammate got a +1/+1 counter");
}

/// CR 702.177 — Exhaust: Camera Launcher's exhaust ability resolves once
/// (a +1/+1 counter + a Thopter), then can never be activated again — not
/// even after turn cleanup clears the once-per-turn budget.
#[test]
fn cr_702_177_exhaust_ability_activates_only_once_per_game() {
    let mut g = two_player_game();
    let cam = g.add_card_to_battlefield(0, catalog::camera_launcher());
    g.clear_sickness(cam);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cam, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("first exhaust activation");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cam).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "got a +1/+1 counter");
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Thopter"),
        "created a Thopter token");

    // Same turn → rejected.
    g.players[0].mana_pool.add_colorless(3);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: cam, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "exhaust ability can't be activated twice");

    // Turn cleanup clears once-per-turn state, but exhaust persists (per game).
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == cam) {
        c.clear_end_of_turn_effects();
    }
    g.players[0].mana_pool.add_colorless(3);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: cam, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "exhaust stays spent across turns");
}

/// Hazard of the Dunes' exhaust adds three +1/+1 counters once; a 4/4 → 7/7.
#[test]
fn cr_702_177_hazard_of_the_dunes_exhaust_counters_once() {
    let mut g = two_player_game();
    let h = g.add_card_to_battlefield(0, catalog::hazard_of_the_dunes());
    g.clear_sickness(h);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::ActivateAbility {
        card_id: h, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("exhaust");
    drain_stack(&mut g);
    let c = g.battlefield_find(h).unwrap();
    assert_eq!((c.power(), c.toughness()), (7, 7), "4/4 + three +1/+1 = 7/7");
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(6);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: h, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "exhaust can't repeat");
}

/// Pacesetter Paragon's exhaust adds a +1/+1 counter and grants double strike
/// until end of turn.
#[test]
fn cr_702_177_pacesetter_paragon_exhaust_grants_double_strike() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::pacesetter_paragon());
    g.clear_sickness(p);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: p, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("exhaust");
    drain_stack(&mut g);
    let c = g.computed_permanent(p).unwrap();
    assert_eq!((c.power, c.toughness), (3, 4), "got a +1/+1 counter");
    assert!(c.keywords.contains(&Keyword::DoubleStrike), "gained double strike EOT");
}

/// Greenbelt Guardian's non-exhaust {G} ability grants trample repeatedly;
/// its exhaust counter ability fires only once.
#[test]
fn cr_702_177_greenbelt_guardian_repeatable_and_exhaust_abilities() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let gg = g.add_card_to_battlefield(0, catalog::greenbelt_guardian());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(gg);
    // Non-exhaust ability (index 0): grant trample to the bear, twice.
    for _ in 0..2 {
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: gg, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
        }).expect("repeatable trample grant");
        drain_stack(&mut g);
    }
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample));
    // Exhaust ability (index 1): +3/+3 once.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gg, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("exhaust counters");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(gg).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Prowcatcher Specialist's exhaust adds two +1/+1 counters (2/1 → 4/3).
#[test]
fn cr_702_177_prowcatcher_specialist_exhaust() {
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::prowcatcher_specialist());
    g.clear_sickness(p);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: p, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("exhaust");
    drain_stack(&mut g);
    let c = g.battlefield_find(p).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 3));
}

/// Keen Buccaneer's exhaust loots (draw then discard) and adds a +1/+1
/// counter; net hand size unchanged, the creature grows once.
#[test]
fn cr_702_177_keen_buccaneer_exhaust_loots_and_grows() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to discard
    let k = g.add_card_to_battlefield(0, catalog::keen_buccaneer());
    g.clear_sickness(k);
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: k, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("exhaust");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "draw 1 then discard 1 nets even");
    assert_eq!(g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Barkshell Blessing conspired pumps the same target twice (+4/+4 total).
#[test]
fn conspire_barkshell_blessing_pumps_twice() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::barkshell_blessing());
    g.players[0].mana_pool.add(Color::Green, 1);
    let c0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpellConspire {
        card_id: id, conspire_creatures: [c0, c1],
        target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("conspire Barkshell");
    drain_stack(&mut g);
    // 2/2 + (+2/+2)×2 = 6/6.
    let t = g.computed_permanent(target).unwrap();
    assert_eq!((t.power, t.toughness), (6, 6), "pumped by original + copy");
}

/// Skystreak Engineer's exhaust adds two +1/+1 counters (1/3 → 3/5), once.
#[test]
fn cr_702_177_skystreak_engineer_exhaust() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::skystreak_engineer());
    g.clear_sickness(s);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("exhaust");
    drain_stack(&mut g);
    let c = g.battlefield_find(s).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 5));
}

/// Mai, Jaded Edge's exhaust puts a double-strike counter on her.
#[test]
fn cr_702_177_mai_jaded_edge_double_strike_counter() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let m = g.add_card_to_battlefield(0, catalog::mai_jaded_edge());
    g.clear_sickness(m);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: m, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("exhaust");
    drain_stack(&mut g);
    assert!(g.computed_permanent(m).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "gained a double strike counter");
}

/// Stampeding Scurryfoot's exhaust adds a +1/+1 counter and makes a 3/3
/// Elephant, once.
#[test]
fn cr_702_177_stampeding_scurryfoot_exhaust() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::stampeding_scurryfoot());
    g.clear_sickness(s);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("exhaust");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(s).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Elephant"
        && (c.power(), c.toughness()) == (3, 3)), "made a 3/3 Elephant");
}

/// Mindspring Merfolk's X-cost exhaust draws X and counters each Merfolk you
/// control (including itself), once.
#[test]
fn cr_702_177_mindspring_merfolk_x_draw_and_counters() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let m = g.add_card_to_battlefield(0, catalog::mindspring_merfolk());
    let other = g.add_card_to_battlefield(0, catalog::mindspring_merfolk());
    g.clear_sickness(m);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2); // X = 2
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: m, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: Some(2),
    }).expect("exhaust X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew X=2");
    assert_eq!(g.battlefield_find(m).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "each Merfolk you control got a counter");
}

/// CR 702.48 — Offering. Patron of the Akki ({4}{R}{R}) cast via Goblin
/// offering: sacrifice a {1}{R} Goblin, the cost drops by its whole mana
/// cost (color included) to {3}{R}, and the Patron resolves.
#[test]
fn cr_702_48_offering_reduces_cost_by_sacrificed_creature() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::goblin_instigator()); // {1}{R}
    let id = g.add_card_to_hand(0, catalog::patron_of_the_akki());
    // {3}{R} = the reduced offering cost (full {4}{R}{R} minus the Goblin's {1}{R}).
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("offering cast for {3}{R} after sacrificing a {1}{R} Goblin");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == fodder), "Goblin sacrificed to offering");
    assert!(g.battlefield.iter().any(|c| c.id == id), "Patron resolved");
}

/// Offering is illegal with no creature of the offered type to sacrifice.
#[test]
fn cr_702_48_offering_requires_the_offered_creature_type() {
    let mut g = two_player_game();
    // A non-Goblin creature doesn't satisfy Goblin offering.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::patron_of_the_akki());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    let r = g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(r.is_err(), "no Goblin → offering rejected");
}

// ── CR 508.1a — "can't attack unless you've cast a creature spell this turn" ──

/// Goblin Cohort can't be declared as an attacker until its controller has
/// cast a creature spell this turn; once one resolves, the lock lifts.
#[test]
fn cr_508_1a_goblin_cohort_gated_on_creature_cast() {
    let mut g = two_player_game();
    let cohort = g.add_card_to_battlefield(0, catalog::goblin_cohort());
    g.clear_sickness(cohort);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    // No creature cast this turn → declaration rejected.
    let err = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: cohort, target: AttackTarget::Player(1),
    }]));
    assert!(matches!(err, Err(GameError::CannotAttack(_))), "locked before a creature cast");

    // Pretend a creature spell resolved this turn, then it can attack.
    g.players[0].creatures_cast_this_turn = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: cohort, target: AttackTarget::Player(1),
    }])).expect("attacks once a creature spell has been cast");
    assert!(g.attacking().iter().any(|a| a.attacker == cohort), "Cohort attacking");
}

// ── CR 502.3 — Hokori: lands don't untap; one is freed at each upkeep ─────────

/// Under Hokori the untap step leaves all lands tapped; the each-upkeep trigger
/// untaps exactly one land the active player controls.
#[test]
fn cr_502_3_hokori_locks_lands_and_frees_one_at_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hokori_dust_drinker());
    let l1 = g.add_card_to_battlefield(0, catalog::island());
    let l2 = g.add_card_to_battlefield(0, catalog::island());
    g.battlefield_find_mut(l1).unwrap().tapped = true;
    g.battlefield_find_mut(l2).unwrap().tapped = true;
    g.do_untap();
    assert_eq!([l1, l2].iter().filter(|id| g.battlefield_find(**id).unwrap().tapped).count(), 2,
        "untap step frees no land under Hokori");
    // Fire the each-player-upkeep trigger directly.
    let trig = catalog::hokori_dust_drinker().triggered_abilities[0].effect.clone();
    let hok = g.battlefield.iter().find(|c| c.definition.name == "Hokori, Dust Drinker").unwrap().id;
    let ctx = crate::game::effects::EffectContext::for_trigger(hok, 0, None, 0);
    g.active_player_idx = 0;
    g.resolve_effect(&trig, &ctx).unwrap();
    assert_eq!([l1, l2].iter().filter(|id| g.battlefield_find(**id).unwrap().tapped).count(), 1,
        "the upkeep trigger frees exactly one land");
}

// ── CR 603.2 — global "whenever a creature becomes the target" trigger ────────

/// Horobi fires its global became-the-target trigger for an opponent's creature
/// too (not just its own), destroying it (CR 603.2 + AnyPlayer scope).
#[test]
fn cr_603_2_horobi_fires_for_any_creature_targeted() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::horobi_deaths_wail()); // opponent's Horobi
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pump = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    // I target my own creature with a pump; Horobi still destroys it.
    g.perform_action(GameAction::CastSpell {
        card_id: pump, target: Some(Target::Permanent(mine)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("pump my own bear");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(mine).is_none(), "Horobi destroyed the targeted creature");
}

// ── CR 704.5j — legend rule + Brothers Yamazaki pair exception ────────────────

/// Two same-name legendaries one player controls collapse to one (CR 704.5j).
#[test]
fn cr_704_5j_legend_rule_keeps_one_same_name_legend() {
    let mut g = two_player_game();
    let l1 = g.add_card_to_battlefield(0, catalog::kasmina_enigma_sage());
    let l2 = g.add_card_to_battlefield(0, catalog::kasmina_enigma_sage());
    g.check_state_based_actions();
    let kept: Vec<CardId> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Kasmina, Enigma Sage").map(|c| c.id).collect();
    assert_eq!(kept.len(), 1, "legend rule keeps exactly one");
    assert!(kept == vec![l2] || kept == vec![l1], "one of the two survives");
}

/// Brothers Yamazaki: exactly two coexist (CR 704.5j exception); a third
/// re-engages the legend rule.
#[test]
fn cr_704_5j_brothers_yamazaki_pair_exempt() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::brothers_yamazaki());
    g.add_card_to_battlefield(0, catalog::brothers_yamazaki());
    g.check_state_based_actions();
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Brothers Yamazaki").count(),
        2, "exactly two Brothers Yamazaki coexist"
    );
    // A third triggers the legend rule again (3 != 2, exception lapses).
    g.add_card_to_battlefield(0, catalog::brothers_yamazaki());
    g.check_state_based_actions();
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Brothers Yamazaki").count(),
        1, "a third re-engages the legend rule"
    );
}

// ── CR 702.36 — Fear ──────────────────────────────────────────────────────────

/// A Fear creature can be blocked only by artifact and/or black creatures.
#[test]
fn cr_702_36_fear_blockable_only_by_artifact_or_black() {
    let attacker = catalog::nezumi_cutthroat(); // 2/1 Fear
    let attacker_kws = &attacker.keywords;
    let assert_block = |def: crate::card::CardDefinition, expect: bool, why: &str| {
        let mut g = two_player_game();
        let blk = g.add_card_to_battlefield(1, def);
        let inst = g.battlefield_find(blk).unwrap().clone();
        let cp = g.computed_permanent(blk).unwrap();
        assert_eq!(
            crate::game::can_block_attacker_computed(&inst, &cp, attacker_kws, &[], 2),
            expect, "{why}"
        );
    };
    assert_block(catalog::grizzly_bears(), false, "green creature can't block Fear");
    assert_block(catalog::ornithopter(), true, "artifact creature can block Fear");
    assert_block(catalog::nezumi_cutthroat(), true, "black creature can block Fear");
}

// ── CR 712.4 — a transformed DFC reverts to its front face off-battlefield ────

/// A transformed double-faced permanent that dies is its front face in the
/// graveyard (CR 712.4).
#[test]
fn cr_712_4_transformed_dfc_reverts_to_front_in_graveyard() {
    let mut g = two_player_game();
    let delver = g.add_card_to_battlefield(0, catalog::delver_of_secrets());
    let mut events = Vec::new();
    g.transform_permanent(delver, &mut events);
    assert!(g.battlefield_find(delver).unwrap().transformed, "transformed on battlefield");
    g.remove_from_battlefield_to_graveyard_raw(delver);
    let in_gy = g.players[0].graveyard.iter().find(|c| c.id == delver).expect("in graveyard");
    assert!(!in_gy.transformed, "reverts off the battlefield");
    assert_eq!(in_gy.definition.name, "Delver of Secrets", "front face restored");
}

// ── CR 711 + 704.5j — a flipped legendary obeys the legend rule ───────────────

/// When a flip card flips into its Legendary face (Azamuki) and the controller
/// already controls one, the legend-rule SBA collapses them to one — exercising
/// the supertype change the flip applies in place.
#[test]
fn cr_711_flipped_legendary_obeys_legend_rule() {
    use crate::card::CounterType;
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // One already-flipped Azamuki (flip a Cunning Bandit), plus a second.
    let b1 = g.add_card_to_battlefield(0, catalog::cunning_bandit());
    let b2 = g.add_card_to_battlefield(0, catalog::cunning_bandit());
    for b in [b1, b2] {
        g.battlefield_find_mut(b).unwrap().add_counters(CounterType::Ki, 2);
    }
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true), DecisionAnswer::Bool(true),
    ]));
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(&mut g);
    g.check_state_based_actions();
    let azamuki = g.battlefield.iter()
        .filter(|c| c.definition.name == "Azamuki, Treachery Incarnate").count();
    assert_eq!(azamuki, 1, "two flipped Azamuki collapse to one under the legend rule");
}

/// CR 702.16c — a permanent with protection from red can't be targeted by an
/// activated ability whose source is red (Cunning Sparkmage's tap-ping).
#[test]
fn cr_702_16c_ability_cant_target_protection_from_source_color() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::cunning_sparkmage()); // red
    g.clear_sickness(mage);
    let priest = g.add_card_to_battlefield(1, catalog::soltari_priest()); // pro-red
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Permanent(priest)),
        additional_targets: Vec::new(), x_value: None,
    });
    assert!(err.is_err(), "a red source's ability can't target protection from red");
    // A non-protected creature is a fine target.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("can ping an unprotected creature");
}

// ── CR 704.5n / 303.4f — illegally-attached Aura SBA ──────────────────────────

/// An "enchant creature you control" Aura whose host changes controllers is
/// put into its owner's graveyard (CR 704.5n).
#[test]
fn cr_704_5n_you_control_aura_falls_off_on_control_change() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::starlit_mantle());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Starlit Mantle on own creature");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(aura).unwrap().attached_to, Some(bear), "attached");
    // The opponent gains control of the bear; the "you control" Aura is now
    // illegally attached and is put into its owner's graveyard.
    g.battlefield_find_mut(bear).unwrap().controller = 1;
    g.check_state_based_actions();
    assert!(g.battlefield_find(aura).is_none(), "Aura left the battlefield");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == aura), "into owner's graveyard");
}

/// A plain "enchant creature" Aura stays attached when its host merely changes
/// controllers (no control restriction to violate).
#[test]
fn cr_704_5n_plain_creature_aura_survives_control_change() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::aspect_of_manticore());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Aspect of Manticore");
    drain_stack(&mut g);
    g.battlefield_find_mut(bear).unwrap().controller = 1;
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(aura).unwrap().attached_to, Some(bear),
        "plain enchant-creature Aura stays put");
}

// ── CR 506.4 — remove from combat ─────────────────────────────────────────────

/// Labyrinth of Skophos removes a declared attacker from combat (CR 506.4).
#[test]
fn cr_506_4_labyrinth_removes_attacker_from_combat() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let labyrinth = g.add_card_to_battlefield(0, catalog::labyrinth_of_skophos());
    g.clear_sickness(labyrinth);
    g.set_attacking(vec![Attack { attacker, target: AttackTarget::Player(0) }]);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: labyrinth, ability_index: 1, target: Some(Target::Permanent(attacker)),
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("activate Labyrinth removal");
    drain_stack(&mut g);
    assert!(!g.attacking().iter().any(|a| a.attacker == attacker),
        "attacker removed from combat");
}

// ── CR 606 — opponent loyalty-ability tax ─────────────────────────────────────

/// Eidolon of Obstruction makes an opponent's loyalty ability cost {1} more.
#[test]
fn cr_606_eidolon_taxes_opponent_loyalty() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::eidolon_of_obstruction());
    let karn = g.add_card_to_battlefield(1, catalog::karn_scion_of_urza());
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    // No mana → the {1} tax can't be paid, activation is rejected.
    assert!(g.activate_loyalty_ability(karn, 0, None, None).is_err(),
        "untaxed activation blocked with no mana");
    // Pay the tax and it goes through.
    g.players[1].mana_pool.add_colorless(1);
    assert!(g.activate_loyalty_ability(karn, 0, None, None).is_ok(),
        "activation succeeds once the {{1}} tax is paid");
}

// ── CR 509.1b — block restrictions ────────────────────────────────────────────

/// Temple Thief can't be blocked by enchantment creatures (CR 509.1b).
#[test]
fn cr_509_1b_temple_thief_cant_be_blocked_by_enchantment_creatures() {
    let mut g = two_player_game();
    let thief = g.add_card_to_battlefield(0, catalog::temple_thief());
    let ench_creature = g.add_card_to_battlefield(1, catalog::skola_grovedancer()); // ench creature
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(!g.blocker_can_block_attacker(ench_creature, thief), "enchantment creature can't block");
    assert!(g.blocker_can_block_attacker(bear, thief), "plain creature can block");
}

/// Serpent of Yawning Depths can only be blocked by sea creatures (CR 509.1b).
#[test]
fn cr_509_1b_serpent_only_blocked_by_sea_creatures() {
    let mut g = two_player_game();
    let serpent = g.add_card_to_battlefield(0, catalog::serpent_of_yawning_depths());
    let kraken = g.add_card_to_battlefield(1, catalog::nadir_kraken()); // Kraken
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(g.blocker_can_block_attacker(kraken, serpent), "Kraken can block");
    assert!(!g.blocker_can_block_attacker(bear, serpent), "non-sea creature can't");
}

// ── CR 509 (declare blockers) + 702.20 (Vigilance) — Grasping Giant ───────────

fn cr_advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// CR 509.1c — a creature becomes blocked by *each* creature blocking it, so
/// Grasping Giant exiles every blocker (the `Selector::BlockingCreatures`
/// reverse-lookup of the block map).
#[test]
fn cr_509_1c_grasping_giant_exiles_every_blocker() {
    let mut g = two_player_game();
    let giant = g.add_card_to_battlefield(0, catalog::grasping_giant());
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(giant);
    cr_advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: giant, target: AttackTarget::Player(1),
    }])).expect("attack");
    // CR 702.20 — Vigilance: attacking doesn't tap the giant.
    assert!(!g.battlefield_find(giant).unwrap().tapped, "vigilant attacker stays untapped");
    drain_stack(&mut g);
    cr_advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, giant), (b2, giant)])).expect("double block");
    drain_stack(&mut g);
    assert!(g.battlefield_find(b1).is_none() && g.battlefield_find(b2).is_none(), "both blockers exiled");
}

// ── CR 117.7c / 601.2f — cost reductions can't reduce colored pips ─────────────

/// CR 601.2f — Thryx's {1}-off discount applies to the generic part only; a
/// mana-value-5+ spell still needs all its colored pips.
#[test]
fn cr_601_2f_thryx_discount_does_not_pay_colored() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::thryx_the_sudden_storm());
    let dragon = g.add_card_to_hand(0, catalog::shivan_dragon()); // {4}{R}{R} → {3}{R}{R}
    // Only {3}{R} available — the second red pip is unpaid even with the discount.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: dragon, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "discount can't cover a colored pip");
}

/// CR 509.1h / 510.1c — once a creature is blocked it stays blocked even if its
/// blocker leaves; Grasping Giant exiles its lone blocker yet deals no combat
/// damage to the defending player.
#[test]
fn cr_510_1c_grasping_giant_stays_blocked_after_exiling_blocker() {
    let mut g = two_player_game();
    let giant = g.add_card_to_battlefield(0, catalog::grasping_giant());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(giant);
    cr_advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: giant, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    cr_advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, giant)])).expect("block");
    drain_stack(&mut g); // becomes-blocked trigger exiles the blocker
    assert!(g.battlefield_find(blocker).is_none(), "blocker exiled before damage");
    let life_before = g.players[1].life;
    cr_advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before, "blocked attacker deals no damage to the player");
}

// ── CR 613.3 / 613.7 — characteristic-overriding Aura layers ──────────────────
/// Ichthyomorphosis sets the host's base P/T to 0/1 (layer 7b); a +1/+1
/// counter then applies in layer 7c on top → 1/2. The host also loses its
/// printed abilities (layer 6) but the engine reports the new Fish subtype
/// (layer 4).
#[test]
fn cr_613_aura_set_base_pt_then_counter() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flier
    g.battlefield_find_mut(host).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let aura = g.add_card_to_hand(0, catalog::ichthyomorphosis());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(host)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant");
    drain_stack(&mut g);
    let cp = g.computed_permanent(host).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 2), "base 0/1 (7b) + counter (7c)");
    assert!(cp.keywords.is_empty(), "abilities removed (layer 6)");
}

// ── CR 605.1a — board-conditional mana ability stays a mana ability ───────────
/// Ilysian Caryatid's "{T}: add one of any color; two instead if you control a
/// power-4+ creature" resolves without using the stack — its mana is available
/// to pay for a spell in the same action sequence.
#[test]
fn cr_605_conditional_mana_ability_pays_a_spell() {
    let mut g = two_player_game();
    let dork = g.add_card_to_battlefield(0, catalog::ilysian_caryatid());
    g.clear_sickness(dork);
    g.add_card_to_battlefield(0, catalog::craw_wurm()); // power-4+ → makes two
    let spell = g.add_card_to_hand(0, catalog::grizzly_bears()); // {1}{G}
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Color(Color::Green),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: dork, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("mana ability (no stack)");
    assert!(g.stack.is_empty(), "mana abilities don't use the stack (CR 605.3a)");
    assert_eq!(g.players[0].mana_pool.total(), 2, "two mana floated");
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("the floated mana pays {1}{G}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(spell).is_some(), "bear cast off the dork's mana");
}

// ── CR 701.19 — Searching (multi-card) ────────────────────────────────────────
/// Deathbellow War Cry searches for up to four Minotaurs; the count-search
/// chains single picks, and a non-matching card is never offered.
#[test]
fn cr_701_19_search_up_to_n_picks_matches_only() {
    let mut g = two_player_game();
    let m = g.add_card_to_library(0, catalog::rage_scarred_berserker());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::deathbellow_war_cry());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add_colorless(5);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(m)),
        crate::decision::DecisionAnswer::Search(None),
        crate::decision::DecisionAnswer::Search(None),
        crate::decision::DecisionAnswer::Search(None),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(m).is_some(), "Minotaur tutored to battlefield");
    assert!(g.players[0].library.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the non-Minotaur stays in the library");
}

// ── CR 700.5 — Devotion (Altar of the Pantheon bonus flips a God on) ─────────

#[test]
fn cr_700_5_altar_devotion_bonus_flips_god_to_creature() {
    let mut g = two_player_game();
    let heliod = g.add_card_to_battlefield(0, catalog::heliod_god_of_the_sun()); // {3}{W}, threshold 5
    g.add_card_to_battlefield(0, catalog::soul_warden()); // {W}
    g.add_card_to_battlefield(0, catalog::soul_warden());
    g.add_card_to_battlefield(0, catalog::soul_warden()); // 3 + Heliod's W = 4 devotion
    assert!(
        !g.computed_permanent(heliod).unwrap().card_types.contains(&crate::card::CardType::Creature),
        "devotion 4 < 5 — Heliod isn't a creature"
    );
    g.add_card_to_battlefield(0, catalog::altar_of_the_pantheon()); // +1 → 5
    assert!(
        g.computed_permanent(heliod).unwrap().card_types.contains(&crate::card::CardType::Creature),
        "Altar's +1 devotion (CR 700.5) reaches 5 — Heliod is now a creature"
    );
}

// ── CR 615.1 / 702.15g — fog exception still lets exempt creatures lifelink ──

#[test]
fn cr_615_1_inspire_awe_exempt_enchantment_creature_deals_and_lifelinks() {
    let mut g = two_player_game();
    let vanilla = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // fogged
    let eidolon = g.add_card_to_battlefield(0, catalog::hateful_eidolon()); // 1/2 enchantment creature, lifelink
    g.clear_sickness(vanilla);
    g.clear_sickness(eidolon);
    let spell = g.add_card_to_hand(0, catalog::inspire_awe());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Inspire Awe");
    drain_stack(&mut g);
    let opp = g.players[1].life;
    let me = g.players[0].life;
    cr_advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: vanilla, target: AttackTarget::Player(1) },
        Attack { attacker: eidolon, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    cr_advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    drain_stack(&mut g);
    cr_advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, opp - 1, "only the exempt 1/2 enchantment creature connects");
    assert_eq!(g.players[0].life, me + 1, "lifelink scales off the unprevented 1 damage (CR 702.15g)");
}

// ── CR 702.3b — Defender can't attack ───────────────────────────────────────

#[test]
fn cr_702_3b_defender_cannot_be_declared_attacker() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::sylvan_caryatid()); // 0/3 Defender
    g.clear_sickness(wall);
    cr_advance_to(&mut g, TurnStep::DeclareAttackers);
    let err = g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: wall, target: AttackTarget::Player(1) },
    ]));
    assert!(err.is_err(), "a Defender creature can't be declared as an attacker (CR 702.3b)");
}

// ── CR 702.16 — Protection (from each mana value other than N) ───────────────

#[test]
fn cr_702_16_protection_from_mana_value_blocks_targeting() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(1)])); // d3=1 → chosen 2
    let haktos = g.add_card_to_battlefield(0, catalog::haktos_the_unscarred());
    let eff = catalog::haktos_the_unscarred().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(haktos, 0, None, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    // Opponent's Lightning Bolt is mana value 1 (≠ 2) → can't target Haktos.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(haktos)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "a MV-1 spell can't target protection-from-each-MV-other-than-2 (CR 702.16)");
}

// ── CR 509.1b — block restriction from protection-by-mana-value ──────────────

#[test]
fn cr_509_1b_protection_from_mv_restricts_blockers() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(1)])); // d3=1 → chosen 2
    let haktos = g.add_card_to_battlefield(0, catalog::haktos_the_unscarred());
    let eff = catalog::haktos_the_unscarred().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(haktos, 0, None, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    g.clear_sickness(haktos);
    let mv3 = g.add_card_to_battlefield(1, catalog::gray_ogre());     // MV 3
    let mv2 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    g.clear_sickness(mv3);
    g.clear_sickness(mv2);
    g.step = TurnStep::DeclareBlockers;
    g.attacking.push(Attack { attacker: haktos, target: AttackTarget::Player(1) });
    let err = g.perform_action(GameAction::DeclareBlockers(vec![(mv3, haktos)]));
    assert!(err.is_err(), "MV-3 creature can't block Haktos (protection from each MV other than 2)");
    g.block_map.clear();
    let ok = g.perform_action(GameAction::DeclareBlockers(vec![(mv2, haktos)]));
    assert!(ok.is_ok(), "MV-2 creature may block Haktos");
}

// ── CR 704.5g — a creature with toughness 0 is put into its graveyard ────────

#[test]
fn cr_704_5g_zero_toughness_creature_dies() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::MinusOneMinusOne, 2);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "the 0-toughness creature left the battlefield");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear), "put into its owner's graveyard (CR 704.5g)");
}

// ── CR 702.160 — Prototype ──────────────────────────────────────────────────

#[test]
fn cr_702_160_prototype_sets_cost_color_and_size_keeping_abilities() {
    let mut g = two_player_game();
    // Full cast: colorless, full size, keeps abilities (Deathtouch).
    let full = g.add_card_to_hand(0, catalog::goring_warplow());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 6);
    }
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: full, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("full cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(full).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 4));
    assert!(cp.colors.is_empty(), "full-cost prototype is colorless (CR 702.160c)");

    // Prototype cast: colored, smaller, same abilities, smaller mana value.
    let proto = g.add_card_to_hand(0, catalog::goring_warplow());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastPrototype {
        card_id: proto, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("prototype cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(proto).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "prototype size (CR 702.160c)");
    assert_eq!(cp.colors, vec![Color::Black], "prototype color follows its cost");
    assert!(cp.keywords.contains(&crate::card::Keyword::Deathtouch), "abilities/types kept");
}

// ── CR 702.21 — Ward (cost = source's power) ─────────────────────────────────

#[test]
fn cr_702_21_ward_life_equal_to_power_is_paid_or_countered() {
    let mut g = two_player_game();
    let gorger = g.add_card_to_battlefield(0, catalog::phyrexian_fleshgorger()); // 7/5
    // Opponent targets it: Ward—Pay life equal to its power (7).
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    let before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(gorger)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 7, "Ward paid life = source power");
    assert!(g.battlefield_find(gorger).is_some(), "creature survives the 3-damage bolt");
}

// ── CR 603.3d — "This ability triggers only once each turn" ──────────────────

#[test]
fn cr_603_3d_trigger_fires_at_most_once_per_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tocasias_welcome());
    let nid = g.next_id();
    g.players[0].library.push(crate::card::CardInstance::new(nid, catalog::lightning_bolt(), 0));
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let cast_small = |g: &mut GameState| {
        let c = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: c, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(g);
    };
    cast_small(&mut g);
    cast_small(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "the once-each-turn trigger drew only once");
}

// ── CR 702.140 — Mutate ──────────────────────────────────────────────────────

/// CR 702.140b — a mutating creature spell whose target is illegal as it
/// begins resolving ceases to be a mutating creature spell and resolves as a
/// normal creature spell, entering the battlefield on its own.
#[test]
fn cr_702_140b_illegal_host_enters_as_normal_creature() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let recluse = g.add_card_to_hand(0, catalog::glowstone_recluse());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastMutate {
        card_id: recluse, target: host, on_top: true, x_value: None,
    }).expect("cast for mutate");
    // Host leaves before the mutate spell resolves — its target is now illegal.
    g.remove_to_graveyard_with_triggers(host);
    drain_stack(&mut g);
    // The spell entered as its own 2/3 Spider rather than merging.
    let r = g.battlefield_find(recluse).expect("Glowstone Recluse entered on its own");
    assert_eq!((r.power(), r.toughness()), (2, 3));
    assert!(r.mutate_stack.is_empty(), "no merge happened");
}

// ── CR 702.31 — Horsemanship ─────────────────────────────────────────────────

/// CR 702.31b — a creature with horsemanship can't be blocked by creatures
/// without horsemanship (one-directional, like flying).
#[test]
fn cr_702_31_horsemanship_only_blocked_by_horsemanship() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::guan_yu_sainted_warrior()); // Horsemanship
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // no horsemanship
    let acomp = g.computed_permanent(atk).unwrap();
    let binst = g.battlefield_find(blk).unwrap();
    let bcomp = g.computed_permanent(blk).unwrap();
    assert!(!crate::game::can_block_attacker_computed(
        binst, &bcomp, &acomp.keywords, &acomp.colors, acomp.power),
        "a non-horsemanship creature can't block a horsemanship attacker");
}

// ── CR 702.28 — Shadow (reverse direction) ───────────────────────────────────

/// CR 702.28b — a creature *without* shadow can't be blocked by a creature
/// *with* shadow either.
#[test]
fn cr_702_28b_shadow_creature_cant_block_nonshadow() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // no shadow
    let blk = g.add_card_to_battlefield(1, catalog::soltari_priest()); // Shadow
    let acomp = g.computed_permanent(atk).unwrap();
    let binst = g.battlefield_find(blk).unwrap();
    let bcomp = g.computed_permanent(blk).unwrap();
    assert!(!crate::game::can_block_attacker_computed(
        binst, &bcomp, &acomp.keywords, &acomp.colors, acomp.power),
        "a shadow creature can't block a non-shadow attacker");
}

// ── CR 702.12 — Defender ─────────────────────────────────────────────────────

/// CR 702.12b/702.12c — a creature with defender can't attack, but a static
/// ability may let it attack anyway while a condition holds (Drowsing
/// Tyrannodon — `CanAttackIgnoringDefenderWhile`).
#[test]
fn cr_702_12_defender_blocks_attack_unless_overridden() {
    let mut g = two_player_game();
    let dino = g.add_card_to_battlefield(0, catalog::drowsing_tyrannodon());
    g.clear_sickness(dino);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    // 702.12c: defender stops the attack with no qualifying creature.
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dino, target: AttackTarget::Player(1),
    }])).is_err());
    // Override condition met → it may attack despite defender.
    let beater = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.clear_sickness(beater);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dino, target: AttackTarget::Player(1),
    }])).expect("attacks once the static condition is satisfied");
}

// ── CR 603.7e — delayed "when you cast your next spell this turn" ─────────────

/// CR 603.7e — a one-shot delayed trigger set up by a resolving ability fires
/// on the controller's next qualifying spell, with that spell available to the
/// trigger (Vivien's −2 reads the cast spell's mana value).
#[test]
fn cr_603_7e_next_spell_delayed_trigger_sees_the_cast_spell() {
    let mut g = two_player_game();
    let vivien = g.add_card_to_battlefield(0, catalog::vivien_monsters_advocate());
    let small = g.add_card_to_library(0, catalog::grizzly_bears()); // MV2 < MV6
    let big = g.add_card_to_hand(0, catalog::colossal_dreadmaw());   // MV6
    g.decider = Box::new(crate::decision::ScriptedDecider::new(
        [crate::decision::DecisionAnswer::Search(Some(small))],
    ));
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: vivien, ability_index: 1, target: None, x_value: None,
    }).expect("arm the delayed trigger");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast the next spell");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == small),
        "delayed trigger fired on the next cast and read its mana value");
}

// ── CR 122.2 — counters cease to exist on zone change ────────────────────────

/// CR 122.2 — when a permanent leaves the battlefield, its counters cease to
/// exist (verified with a freshly-added counter kind, Bounty).
#[test]
fn cr_122_2_counters_vanish_on_zone_change() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::Bounty, 2);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Bounty), 2);
    g.remove_to_graveyard_with_triggers(bear);
    // Re-entering as a new object carries no counters.
    let again = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.battlefield_find(again).unwrap().counter_count(CounterType::Bounty), 0,
        "counters did not persist across the zone change");
}

// ── CR 702.11e — Hexproof from [color] ───────────────────────────────────────

/// CR 702.11e — a creature with "hexproof from black" can't be targeted by an
/// opponent's black spell, but a white spell targets it fine. Knight of Grace.
#[test]
fn cr_702_11e_hexproof_from_black_blocks_only_black() {
    let mut g = two_player_game();
    let knight = g.add_card_to_battlefield(0, catalog::knight_of_grace());
    // Opponent's black removal can't target it.
    let blade = g.add_card_to_hand(1, catalog::doom_blade());
    g.players[1].mana_pool.add(Color::Black, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: blade, target: Some(Target::Permanent(knight)),
        additional_targets: vec![], mode: None, x_value: None });
    assert!(matches!(err, Err(GameError::TargetHasHexproof(_))), "black blocked, got {err:?}");
    // A white spell is unaffected.
    let swords = g.add_card_to_hand(1, catalog::swords_to_plowshares());
    g.players[1].mana_pool.add(Color::White, 1);
    crate::game::cast_at(&mut g, swords, Target::Permanent(knight));
    assert!(g.battlefield_find(knight).is_none(), "white Swords exiled it");
}

// ── CR 702.165 — Gift ─────────────────────────────────────────────────────────

/// CR 702.165 — promising a Gift bestows the gift on the opponent *before* the
/// spell's other (enhanced) effects, and broadens the resolution accordingly.
#[test]
fn cr_702_165_gift_promised_gives_opponent_and_enhances() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let blast = g.add_card_to_hand(0, catalog::blooming_blast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastGift {
        card_id: blast, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Blooming Blast with gift");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Treasure"),
        "opponent received the promised Treasure");
    assert_eq!(g.players[1].life, 17, "the enhanced effect burned the controller for 3");
}

// ── CR 702.180 — Survival ──────────────────────────────────────────────────────

/// CR 702.180 — Survival triggers at the controller's second main phase while
/// the creature is tapped (the intervening-`if`); the untapped case is covered
/// by `survival_skips_when_untapped`.
#[test]
fn cr_702_180_survival_fires_while_tapped_at_second_main() {
    let mut g = two_player_game();
    let surv = g.add_card_to_battlefield(0, catalog::cautious_survivor());
    g.battlefield_find_mut(surv).unwrap().tapped = true;
    let life = g.players[0].life;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "Survival gained 2 life once tapped");
}

// ── CR 702.183 — Omen ──────────────────────────────────────────────────────────

/// CR 702.183 — casting a card's Omen half resolves the Omen effect and shuffles
/// the card into its owner's library (not the graveyard); the creature stays
/// available to be drawn and cast later.
#[test]
fn cr_702_183_omen_resolves_then_shuffles_into_library() {
    let mut g = two_player_game();
    let regent = g.add_card_to_hand(0, catalog::marang_river_regent());
    // Seed the library so Coil and Catch's draw-three has cards to find.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    // Coil and Catch costs {3}{U}: draw three, discard one.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastOmen {
        card_id: regent, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast the Omen half");
    drain_stack(&mut g);
    // Net hand: -1 (Regent left) +3 draw -1 discard = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew three, discarded one");
    // The Regent itself is shuffled back into the library, not the graveyard.
    assert!(g.players[0].graveyard.iter().all(|c| c.id != regent),
        "Omen card did not go to the graveyard");
    assert!(g.players[0].library.iter().any(|c| c.id == regent),
        "Omen card was shuffled into the library");
    assert_eq!(g.players[0].library.len(), lib_before - 3 + 1, "three drawn, Regent shuffled in");
}

/// CR 702.183 — a countered Omen spell shuffles into its owner's library rather
/// than going to the graveyard.
#[test]
fn cr_702_183_countered_omen_shuffles_into_library() {
    let mut g = two_player_game();
    let regent = g.add_card_to_hand(0, catalog::bloomvine_regent());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastOmen {
        card_id: regent, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Claim Territory");
    // Lift the spell off the stack as if countered (CR 701.5g → 702.183 instead).
    let card = match g.stack.pop().expect("Omen spell on the stack") {
        crate::game::StackItem::Spell { card, .. } => *card,
        _ => unreachable!("top of stack is the Omen spell"),
    };
    assert_eq!(card.id, regent, "Omen spell is on the stack");
    let mut events = Vec::new();
    g.route_to_graveyard(card, &mut events);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != regent),
        "countered Omen did not hit the graveyard");
    assert!(g.players[0].library.iter().any(|c| c.id == regent),
        "countered Omen was shuffled into the library");
}

// ── CR 508.3a — put onto the battlefield attacking ─────────────────────────────

/// CR 508.3a — Alesha's attack trigger reanimates a power-≤2 creature card
/// *tapped and attacking* via `Effect::JoinCombatAttacking`.
#[test]
fn cr_508_3a_alesha_reanimates_tapped_and_attacking() {
    let mut g = two_player_game();
    let alesha = g.add_card_to_battlefield(0, catalog::alesha_who_smiles_at_death());
    g.clear_sickness(alesha);
    // A 2/2 bear in the graveyard is a legal reanimate target (power ≤ 2).
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    // Accept the optional "pay {W/B}{W/B}" trigger (AutoDecider declines by default).
    g.decider = Box::new(crate::decision::ScriptedDecider::new(
        [crate::decision::DecisionAnswer::Bool(true)],
    ));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    // Float {W}{W} to pay the {W/B}{W/B} attack trigger.
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: alesha, target: AttackTarget::Player(1),
    }])).expect("Alesha attacks");
    drain_stack(&mut g);
    let reanimated = g.battlefield_find(bear).expect("bear returned to the battlefield");
    assert!(reanimated.tapped, "reanimated creature enters tapped (CR 508.3a)");
    assert!(g.attacking.iter().any(|a| a.attacker == bear), "and joins combat attacking");
}

/// Skimming Strike (Dirgur Island Dragon's Omen) taps a creature and draws.
#[test]
fn omen_skimming_strike_taps_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dragon = g.add_card_to_hand(0, catalog::dirgur_island_dragon());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastOmen {
        card_id: dragon, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Skimming Strike");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "target creature tapped");
    // -1 (Dragon left hand) +1 draw = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "drew a card after tapping");
    assert!(g.players[0].library.iter().any(|c| c.id == dragon), "Dragon shuffled back");
}

/// Exude Toxin ({X}{B}{B}, Scavenger Regent's Omen) gives each non-Dragon
/// creature -X/-X; Dragons are spared.
#[test]
fn omen_exude_toxin_spares_dragons() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 non-Dragon
    let dragon = g.add_card_to_battlefield(1, catalog::bloomvine_regent()); // 4/5 Dragon
    let regent = g.add_card_to_hand(0, catalog::scavenger_regent());
    // {X}{B}{B} with X=2 → {2}{B}{B}.
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastOmen {
        card_id: regent, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast Exude Toxin for X=2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the 2/2 non-Dragon died to -2/-2");
    assert!(g.battlefield_find(dragon).is_some(), "the Dragon was spared");
}

/// Charring Bite (Twinmaw Stormbrood's Omen) deals 5 to a creature without
/// flying — enough to kill a ground blocker, while a flyer is not a legal target.
#[test]
fn omen_charring_bite_burns_ground_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let brood = g.add_card_to_hand(0, catalog::twinmaw_stormbrood());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastOmen {
        card_id: brood, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Charring Bite at the ground bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Charring Bite's 5 damage killed the 2/2 bear");
    assert!(g.players[0].library.iter().any(|c| c.id == brood),
        "Twinmaw Stormbrood shuffled back after its Omen resolved");
}

/// Roost Seek (Sagu Wildling's Omen) tutors a basic land into hand.
#[test]
fn omen_roost_seek_tutors_basic_land() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let wildling = g.add_card_to_hand(0, catalog::sagu_wildling());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(crate::decision::ScriptedDecider::new(
        [crate::decision::DecisionAnswer::Search(Some(forest))],
    ));
    g.perform_action(GameAction::CastOmen {
        card_id: wildling, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Roost Seek");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "basic Forest tutored to hand");
    assert!(g.players[0].library.iter().any(|c| c.id == wildling), "Sagu Wildling shuffled back");
}

/// CR 701.52 — Nesting Instinct (Pearl Lake Warden's Omen) *seeks* a land card
/// at random and puts it onto the battlefield (no player choice of which land).
#[test]
fn omen_nesting_instinct_seeks_land_to_battlefield() {
    let mut g = two_player_game();
    // Library has only one land among non-land filler, so the random seek must
    // find that land deterministically.
    let island = g.add_card_to_library(0, catalog::island());
    for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let warden = g.add_card_to_hand(0, catalog::pearl_lake_warden());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastOmen {
        card_id: warden, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Nesting Instinct");
    drain_stack(&mut g);
    assert!(g.battlefield_find(island).is_some(), "seeked land entered the battlefield");
    assert!(g.players[0].library.iter().any(|c| c.id == warden), "Pearl Lake Warden shuffled back");
}

/// Chilling Screech (Runescale Stormbrood's Omen) counters a mana-value-2 spell
/// but can't target a higher-cost one.
#[test]
fn omen_chilling_screech_counters_low_mv_spell() {
    let mut g = two_player_game();
    // Opponent casts a 2-mana bear; it's on the stack.
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears()); // {1}{G}, MV 2
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts the bear");
    // We hold the Stormbrood and cast Chilling Screech at the bear spell.
    let brood = g.add_card_to_hand(0, catalog::runescale_stormbrood());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastOmen {
        card_id: brood, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Chilling Screech at the MV-2 spell");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "the MV-2 spell was countered");
    assert!(g.players[0].library.iter().any(|c| c.id == brood), "Runescale Stormbrood shuffled back");
}

/// Signaling Roar (Riling Dawnbreaker's Omen) mints a 2/2 white Soldier token.
#[test]
fn omen_signaling_roar_makes_soldier() {
    let mut g = two_player_game();
    let dawn = g.add_card_to_hand(0, catalog::riling_dawnbreaker());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastOmen {
        card_id: dawn, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Signaling Roar");
    drain_stack(&mut g);
    let token = g.battlefield.iter()
        .find(|c| c.definition.name == "Soldier" && c.controller == 0)
        .expect("a Soldier token was created");
    assert_eq!((token.definition.power, token.definition.toughness), (2, 2), "2/2 Soldier");
}

/// CR 702.182 — Job Select: an Equipment with job select enters, mints a 1/1
/// Hero token, and attaches itself to it.
#[test]
fn cr_702_182_job_select_mints_hero_and_attaches() {
    let mut g = two_player_game();
    let fist = g.add_card_to_hand(0, catalog::monks_fist());
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: fist, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Monk's Fist");
    drain_stack(&mut g);
    let hero = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Hero" && c.controller == 0)
        .expect("a 1/1 Hero token was created");
    let hero_id = hero.id;
    assert_eq!((hero.definition.power, hero.definition.toughness), (1, 1), "1/1 Hero");
    // The Equipment attached itself to the Hero, granting +1/+0 → 2/1.
    let equip = g.battlefield_find(fist).expect("equipment on battlefield");
    assert_eq!(equip.attached_to, Some(hero_id), "Monk's Fist attached to the Hero");
    assert_eq!(g.computed_permanent(hero_id).unwrap().power, 2, "Hero gets +1/+0 from Monk's Fist");
}

/// CR 702.187 — Mayhem: a card discarded this turn may be cast from the
/// graveyard for its mayhem cost, then is exiled when it leaves the stack.
#[test]
fn cr_702_187_mayhem_casts_discarded_card_then_exiles() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let bolt = g.add_card_to_hand(0, catalog::electros_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Discard the Bolt this turn — that's what unlocks its Mayhem cast.
    let mut events = Vec::new();
    g.discard_card(0, bolt, &mut events);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "Bolt is in the graveyard");
    // Pay the Mayhem cost {1}{R} and cast it from the graveyard at the bear.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastMayhem {
        card_id: bolt, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Electro's Bolt via Mayhem");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "Bolt's 4 damage killed the 2/2 bear");
    // CR 702.187 exile-after: the Mayhem spell goes to exile, not the graveyard.
    assert!(g.exile.iter().any(|c| c.id == bolt), "Mayhem-cast spell was exiled");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != bolt), "not in the graveyard");
}

/// A Mayhem cast is illegal if the card wasn't discarded this turn.
#[test]
fn cr_702_187_mayhem_blocked_without_discard_this_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Bolt sits in the graveyard but was NOT discarded this turn.
    let bolt = g.add_card_to_graveyard(0, catalog::electros_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let r = g.perform_action(GameAction::CastMayhem {
        card_id: bolt, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(r.is_err(), "Mayhem can't cast a card not discarded this turn");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "Bolt stays in the graveyard");
}

/// CR 702.188 — Web-slinging: cast Spider-Man, Web-Slinger for {W} by returning
/// a tapped creature you control to its owner's hand instead of paying {2}{W}.
#[test]
fn cr_702_188_web_slinging_returns_tapped_creature() {
    let mut g = two_player_game();
    // A tapped creature to bounce.
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == fodder).unwrap().tapped = true;
    let spider = g.add_card_to_hand(0, catalog::spider_man_web_slinger());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1); // only {W} — the web-slinging cost
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: spider, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("web-sling Spider-Man for {W} + bounce a tapped creature");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == spider), "Spider-Man resolved onto the battlefield");
    assert!(g.battlefield_find(fodder).is_none(), "the tapped creature was returned");
    assert!(g.players[0].hand.iter().any(|c| c.id == fodder), "bounced to its owner's hand");
}

/// Web-slinging is illegal with no tapped creature to return.
#[test]
fn cr_702_188_web_slinging_requires_a_tapped_creature() {
    let mut g = two_player_game();
    // An untapped creature doesn't satisfy the return-a-tapped-creature cost.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spider = g.add_card_to_hand(0, catalog::spider_man_web_slinger());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    let r = g.perform_action(GameAction::CastSpellAlternative {
        card_id: spider, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(r.is_err(), "no tapped creature → web-slinging rejected");
}

/// CR 702.187 — the "if this spell's mayhem cost was paid, …" rider. Cast for
/// the mayhem cost and only opponents' creatures get -2/-2 (Sandman's Quicksand).
#[test]
fn cr_702_187_mayhem_rider_branches_on_mayhem_cast() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::sandmans_quicksand());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let mut events = Vec::new();
    g.discard_card(0, spell, &mut events);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastMayhem {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sandman's Quicksand via Mayhem");
    drain_stack(&mut g);
    // Mayhem cast → only the opponent's creature took -2/-2.
    assert!(g.battlefield_find(theirs).is_none(), "opponent's bear died to -2/-2");
    assert!(g.battlefield_find(mine).is_some(), "my bear was spared by the rider");
}

/// Cast normally (from hand), the rider doesn't fire — all creatures get -2/-2.
#[test]
fn cr_702_187_mayhem_rider_off_when_cast_normally() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::sandmans_quicksand());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("hard-cast Sandman's Quicksand");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "opponent's bear died");
    assert!(g.battlefield_find(mine).is_none(), "my bear died too — both swept");
}

/// CR 702.180 — Harmonize: cast a card from the graveyard for its harmonize
/// cost, tapping a creature to reduce the cost by its power; the spell is
/// exiled after resolving.
#[test]
fn cr_702_180_harmonize_tap_discount_then_exile() {
    let mut g = two_player_game();
    // Channeled Dragonfire (Harmonize {5}{R}{R}) waits in the graveyard.
    let spell = g.add_card_to_graveyard(0, catalog::channeled_dragonfire());
    // A 5/5 to tap: {5}{R}{R} − 5 = {R}{R}.
    let big = g.add_card_to_battlefield(0, catalog::durkwood_baloth());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 2);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastHarmonize {
        card_id: spell,
        tap_creature: Some(big),
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("cast Channeled Dragonfire via Harmonize for {R}{R}");
    assert!(g.battlefield_find(big).unwrap().tapped, "the discounting creature is tapped");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 2, "dealt 2 to the opponent");
    // CR 702.180 exile-after.
    assert!(g.exile.iter().any(|c| c.id == spell), "Harmonize spell was exiled");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != spell), "not in graveyard");
}

/// Without tapping a creature, the full harmonize cost must be paid.
#[test]
fn cr_702_180_harmonize_no_tap_requires_full_cost() {
    let mut g = two_player_game();
    let spell = g.add_card_to_graveyard(0, catalog::channeled_dragonfire());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Only {R}{R} available — not enough for the full {5}{R}{R}.
    g.players[0].mana_pool.add(Color::Red, 2);
    let r = g.perform_action(GameAction::CastHarmonize {
        card_id: spell, tap_creature: None,
        target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(r.is_err(), "can't pay {{5}}{{R}}{{R}} with only {{R}}{{R}} and no tap");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == spell), "spell stays in graveyard");
}

/// Whirlwing Stormbrood's static lets its controller cast sorceries at instant
/// speed (here: on the opponent's turn with a non-empty stack context).
#[test]
fn whirlwing_grants_sorcery_flash_timing() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::whirlwing_stormbrood());
    // A sorcery in hand that would normally be sorcery-speed-only.
    let bolt = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    // It's the opponent's turn (seat 1 active); seat 0 holds priority.
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Whirlwing's static lets the sorcery be cast at instant speed");
    assert!(g.stack.iter().any(|si| matches!(si,
        crate::game::StackItem::Spell { card, .. } if card.id == bolt)),
        "the sorcery is on the stack");
}

// ── CR 120.6 — "loses half their life, rounded up" ─────────────────────────────

/// A "lose half your life, rounded up" effect rounds an odd total upward
/// (CR 120.6 / 119.5). Unstoppable Slasher's combat trigger drives it.
#[test]
fn cr_120_6_lose_half_life_rounds_up() {
    let mut g = two_player_game();
    let slasher = g.add_card_to_battlefield(0, catalog::unstoppable_slasher());
    g.clear_sickness(slasher);
    g.players[1].life = 15;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: slasher, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    // 15 - 2 (combat) = 13; half rounded up = 7; 13 - 7 = 6.
    assert_eq!(g.players[1].life, 6, "odd life halved and rounded up");
}

// ── CR 509.1b — fixed-threshold "can't be blocked by power N or less" ──────────

/// CR 509.1b — Questing Beast's evasion gates on the blocker's *computed*
/// power against a fixed threshold (2), not the attacker's power.
#[test]
fn cr_509_1b_power_threshold_block_restriction() {
    let mut g = two_player_game();
    let qb = g.add_card_to_battlefield(0, catalog::questing_beast());
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // power 2
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // power 4
    assert!(!g.blocker_can_block_attacker(small, qb), "power-2 blocker is illegal");
    assert!(g.blocker_can_block_attacker(big, qb), "power-4 blocker is legal");
}

// ── CR 601.2c — "up to N target" spells accept fewer targets ───────────────────

/// CR 601.2c — a "put a +1/+1 counter on each of up to two target creatures"
/// spell (Gird for Battle) is legal with a single target and still resolves.
#[test]
fn cr_601_2c_up_to_two_targets_accepts_one() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::gird_for_battle());
    g.players[0].mana_pool.add(Color::White, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable with a single target");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1),
        "the single chosen creature got its counter"
    );
}

// ── CR 601.2f / 117.7c — target-conditional cost reduction ────────────────────

/// CR 601.2f — Ride's End ("costs {3} less if it targets a tapped permanent")
/// is castable for {1}{W} against a tapped creature, but its full {4}{W} stands
/// against an untapped one. The colored {W} pip is never reduced (117.7c).
#[test]
fn cr_601_2f_rides_end_target_conditional_reduction() {
    // Tapped target → discounted, castable for {1}{W}.
    let mut g = two_player_game();
    let tapped = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    let re = g.add_card_to_hand(0, catalog::rides_end());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crate::game::cast_at(&mut g, re, Target::Permanent(tapped));
    assert!(g.exile.iter().any(|c| c.id == tapped), "discounted exile of a tapped permanent");

    // Untapped target → no discount; {1}{W} is not enough for {4}{W}.
    let mut g = two_player_game();
    let untapped = g.add_card_to_battlefield(1, catalog::serra_angel());
    let re = g.add_card_to_hand(0, catalog::rides_end());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: re, target: Some(Target::Permanent(untapped)),
            additional_targets: vec![], mode: None, x_value: None,
        }).is_err(),
        "no discount against an untapped target — full cost unpaid"
    );
}

// ── CR 701.13 — Investigate ───────────────────────────────────────────────────

/// CR 701.13 — Hostile Investigator's "whenever a player discards, investigate"
/// (once each turn) creates a Clue artifact token.
#[test]
fn cr_701_13_investigate_makes_a_clue() {
    use crate::card::ArtifactSubtype;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hostile_investigator());
    // A discard (Unburden on seat 0) triggers the investigate.
    let unburden = g.add_card_to_hand(0, catalog::unburden());
    g.add_card_to_hand(0, catalog::mountain());
    g.add_card_to_hand(0, catalog::mountain());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crate::game::cast_at(&mut g, unburden, Target::Player(0));
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0
            && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Clue)),
        "investigate created a Clue token"
    );
}

// ── CR 702.166 — Offspring ────────────────────────────────────────────────────

/// CR 702.166 — paying a creature's Offspring cost makes a 1/1 token copy of it
/// enter when the creature itself enters.
#[test]
fn cr_702_166_offspring_makes_one_one_copy() {
    let mut g = two_player_game();
    let recruit = g.add_card_to_hand(0, catalog::pawpatch_recruit()); // Offspring {2}
    g.players[0].mana_pool.add(Color::Green, 1); // {G} base
    g.players[0].mana_pool.add_colorless(2); // {2} Offspring
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: recruit, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast with Offspring paid");
    drain_stack(&mut g);
    let copies = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Pawpatch Recruit")
        .count();
    assert_eq!(copies, 2, "the creature plus its 1/1 Offspring token");
    let cp = g.compute_battlefield();
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Pawpatch Recruit" && c.is_token
            && cp.iter().find(|p| p.id == c.id).map(|p| (p.power, p.toughness)) == Some((1, 1))),
        "the Offspring copy is a 1/1 token"
    );
}

// ── CR 615.12 (scoped) — Questing Beast: your creatures' combat damage can't be prevented ──

/// CR 615.12 — a Fog (prevent all combat damage this turn) doesn't stop the
/// combat damage of a creature its controller controls while Questing Beast's
/// static is active.
#[test]
fn cr_615_12_questing_beast_combat_damage_ignores_fog() {
    let mut g = two_player_game();
    let qb = g.add_card_to_battlefield(0, catalog::questing_beast()); // 4/4, the static
    g.clear_sickness(qb);
    // A Fog is in effect this turn.
    g.prevent_combat_damage_this_turn = true;
    let opp = g.players[1].life;
    cr_advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: qb, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    cr_advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    drain_stack(&mut g);
    cr_advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, opp - 4, "Questing Beast's combat damage was not prevented");
}

/// Without Questing Beast's static, the same Fog prevents an ordinary
/// attacker's combat damage (control case).
#[test]
fn cr_615_1_fog_prevents_ordinary_attacker() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.prevent_combat_damage_this_turn = true;
    let opp = g.players[1].life;
    cr_advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    cr_advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    drain_stack(&mut g);
    cr_advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, opp, "ordinary combat damage is fogged");
}

// ── CR 122.1c / 701.46a — a stun counter replaces an untap ───────────────────

/// CR 122.1c — a tapped permanent with stun counters stays tapped through the
/// untap step, removing one stun counter each time, then untaps normally.
#[test]
fn cr_122_1c_stun_counter_replaces_untap() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    {
        let inst = g.battlefield_find_mut(c).unwrap();
        inst.tapped = true;
        inst.add_counters(CounterType::Stun, 2);
    }
    g.active_player_idx = 0;
    g.do_untap();
    let inst = g.battlefield_find(c).unwrap();
    assert!(inst.tapped, "stayed tapped, untap replaced");
    assert_eq!(inst.counter_count(CounterType::Stun), 1, "one stun counter removed");
    g.do_untap();
    let inst = g.battlefield_find(c).unwrap();
    assert!(inst.tapped, "still tapped while a stun counter remains");
    assert_eq!(inst.counter_count(CounterType::Stun), 0, "second stun removed");
    g.do_untap();
    assert!(!g.battlefield_find(c).unwrap().tapped, "untaps once stun counters are gone");
}

// ── CR 702.29 — Cycling: pay the cost, discard the card, draw a card ─────────

/// CR 702.29a — cycling a card discards it (to its owner's graveyard) and draws
/// a card, paying the cycling cost.
#[test]
fn cr_702_29_cycling_discards_and_draws() {
    let mut g = two_player_game();
    let card = g.add_card_to_hand(0, catalog::marauding_mako()); // Cycling {2}
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::Cycle { card_id: card, x_value: None }).expect("cycle for {2}");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == card), "cycled card hits the graveyard");
    assert_eq!(g.players[0].hand.len(), hand, "discarded one, drew one (net zero)");
}

// ── CR 701.55 — Face a villainous choice ─────────────────────────────────────

/// CR 701.55a — the chooser performs the chosen option. The bot heuristic
/// picks the lesser self-harm: 2 life lost beats 6 life lost.
#[test]
fn cr_701_55_villainous_choice_picks_lesser_harm() {
    use crate::effect::{Effect, PlayerRef, Selector, Value};
    let mut g = two_player_game();
    g.players[1].life = 20;
    let choice = Effect::VillainousChoice {
        who: Selector::Player(PlayerRef::EachOpponent),
        option_a: Box::new(Effect::LoseLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(6) }),
        option_b: Box::new(Effect::LoseLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(2) }),
    };
    let ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    g.resolve_effect(&choice, &ctx).unwrap();
    assert_eq!(g.players[1].life, 18, "opponent took the 2-life option");
}

/// CR 701.55b — an impossible option may be chosen and does nothing, so the
/// chooser dodges harm (no creature to sacrifice → sacrifice branch is free).
#[test]
fn cr_701_55_villainous_choice_takes_impossible_option_to_dodge() {
    use crate::card::SelectionRequirement;
    use crate::effect::{Effect, PlayerRef, Selector, Value};
    let mut g = two_player_game();
    g.players[1].life = 20;
    // Opponent controls no creature, so the sacrifice option is impossible.
    let choice = Effect::VillainousChoice {
        who: Selector::Player(PlayerRef::EachOpponent),
        option_a: Box::new(Effect::LoseLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(5) }),
        option_b: Box::new(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::You),
            filter: SelectionRequirement::Creature,
            count: Value::Const(1),
        }),
    };
    let ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    g.resolve_effect(&choice, &ctx).unwrap();
    assert_eq!(g.players[1].life, 20, "dodged via the impossible sacrifice option");
}

// ── CR 614.16 — counter-placement replacement ordering ───────────────────────

/// CR 614.16 / 616.1 — Hardened Scales (additive +1) and a Doubling-Counters
/// permanent both replace a +1/+1 placement; the additive applies before the
/// doubling: a 1-counter placement becomes (1+1)*2 = 4.
#[test]
fn cr_614_16_additive_then_doubling_counter_replacement() {
    use crate::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::hardened_scales());
    g.add_card_to_battlefield(0, catalog::witherbloom_pestseed()); // DoubleCounters
    let ctx = crate::game::effects::EffectContext::for_ability(
        crate::card::CardId(0), 0, Some(Target::Permanent(bear)),
    );
    g.resolve_effect(&Effect::AddCounter {
        what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::Const(1),
    }, &ctx).unwrap();
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        4,
        "(1 + Hardened Scales) * Doubling = 4"
    );
}

// ── CR 701.56 — Time travel ──────────────────────────────────────────────────

/// CR 701.56a — time traveling removes a time counter from a suspended card the
/// player owns (the bot heuristic advances its own suspended spells).
#[test]
fn cr_701_56_time_travel_advances_own_suspended_card() {
    use crate::effect::{Effect, PlayerRef};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::durkwood_baloth()); // Suspend 5—{G}
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::Suspend { card_id: id }).expect("suspend");
    let before = g.exile.iter().find(|c| c.id == id).unwrap().counter_count(CounterType::Time);
    assert_eq!(before, 5);
    let ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    g.resolve_effect(&Effect::TimeTravel { who: PlayerRef::You }, &ctx).unwrap();
    let after = g.exile.iter().find(|c| c.id == id).unwrap().counter_count(CounterType::Time);
    assert_eq!(after, 4, "one time counter removed");
}

// ── CR 702.148 — Cleave ──────────────────────────────────────────────────────

/// Casting Path of Peril normally destroys only creatures of mana value 2 or
/// less; casting it for its cleave cost removes the bracketed clause and wipes
/// every creature (the bracket-removal is the broader `effect_override`).
#[test]
fn cr_702_148_cleave_removes_bracketed_clause() {
    // Normal cast: a 5/5 survives.
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // MV 5
    let id = g.add_card_to_hand(0, catalog::path_of_peril());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("normal cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_none(), "MV-2 creature destroyed");
    assert!(g.battlefield_find(big).is_some(), "MV-5 creature spared by the bracketed clause");

    // Cleave cast: the 5/5 dies too.
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::path_of_peril());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cleave cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "cleave wipes all creatures");
}

// ── CR 701.16 — Sacrifice as a reflexive cost ────────────────────────────────

/// `Effect::MaySacrifice` ("you may sacrifice X; if you do, …") declined leaves
/// the board untouched and skips the payoff.
#[test]
fn cr_701_16_reflexive_sacrifice_declined_skips_payoff() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    use crate::effect::{Effect, Value, Selector};
    use crate::card::SelectionRequirement;
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eff = Effect::MaySacrifice {
        description: "sac a creature?".into(),
        filter: SelectionRequirement::Creature,
        count: Value::Const(1),
        then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(5) }),
        else_: None,
    };
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    let ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    g.resolve_effect(&eff, &ctx).unwrap();
    assert!(g.battlefield_find(fodder).is_some(), "nothing sacrificed when declined");
    assert_eq!(g.players[0].life, life, "payoff skipped");
}

// ── EntityMatchesAny — "if a [type] card was exiled this way" ────────────────

/// `Predicate::EntityMatchesAny` is false when the selector is empty, so an
/// "up to one target" rider with no target chosen skips its payoff (Diregraf
/// Scavenger drains nothing when it exiles nothing).
#[test]
fn entity_matches_any_false_on_empty_target() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    g.players[0].life = 20;
    let etb = catalog::diregraf_scavenger().triggered_abilities[0].effect.clone();
    // No target chosen (up-to-one declined) → empty targets.
    let ctx = crate::game::effects::EffectContext::for_trigger(crate::card::CardId(99), 0, None, 0);
    g.resolve_effect(&etb, &ctx).unwrap();
    assert_eq!(g.players[1].life, 20, "no exile → no drain");
    assert_eq!(g.players[0].life, 20, "no exile → no gain");
}

// ── CR 702.34 — Flashback (graveyard-grant via Lier) ──────────────────────────

/// CR 702.34a/d — Lier grants flashback (= mana cost) to instants/sorceries in
/// your graveyard; the recast follows alternative-cost timing and the card is
/// exiled when it leaves the stack.
#[test]
fn cr_702_34_lier_grants_graveyard_flashback_and_exiles() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lier_disciple_of_the_drowned());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    // The grant is visible to the engine helper.
    let inst = g.players[0].graveyard.iter().find(|c| c.id == bolt).unwrap().clone();
    assert!(g.graveyard_flashback_grant(0, &inst).is_some(), "Lier grants flashback");
    g.players[0].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFlashback {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("flashback castable via Lier");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "Bolt resolved for 3");
    assert!(g.exile.iter().any(|c| c.id == bolt), "CR 702.34d — exiled off the stack");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt));
}

/// Without Lier (or printed flashback), a graveyard instant isn't castable.
#[test]
fn cr_702_34_no_grant_no_flashback() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let inst = g.players[0].graveyard.iter().find(|c| c.id == bolt).unwrap().clone();
    assert!(g.graveyard_flashback_grant(0, &inst).is_none(), "no static → no grant");
    g.players[0].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::CastFlashback {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no flashback grant → not castable");
}

// ── CR 702.147 — Decayed ──────────────────────────────────────────────────────

/// CR 702.147a — "Decayed" means "this creature can't block". A decayed Zombie
/// (Ghoulish Procession's token) is denied as a blocker.
#[test]
fn cr_702_147_decayed_creature_cant_block() {
    let mut g = two_player_game();
    // Mint a decayed Zombie under seat 1 via Ghoulish Procession's effect.
    let proc = g.add_card_to_battlefield(1, catalog::ghoulish_procession());
    let trig = catalog::ghoulish_procession().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(proc, 1, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    let zombie = g.battlefield.iter().find(|c| c.is_token).map(|c| c.id).unwrap();
    assert!(g.computed_permanent(zombie).unwrap().keywords.contains(&Keyword::Decayed),
        "token carries Decayed");
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    assert!(!g.blocker_can_block_attacker(zombie, attacker), "CR 702.147 — decayed can't block");
}

// ── CR 702.41 — Affinity ──────────────────────────────────────────────────────

/// CR 702.41a — "Affinity for [text]" reduces a spell's cost by {1} per matching
/// permanent. Millicent's Affinity for Spirits discounts the generic pips.
#[test]
fn cr_702_41_affinity_for_spirits_reduces_cost() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crate::card::CardInstance::new(g.next_id(), catalog::millicent_restless_revenant(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "no Spirits → no discount");
    g.add_card_to_battlefield(0, catalog::millicent_restless_revenant()); // a Spirit
    g.add_card_to_battlefield(0, catalog::millicent_restless_revenant());
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 2, "two Spirits → 2 generic off");
}

// ── CR 702.149 — Training ────────────────────────────────────────────────────

/// CR 702.149a — "Whenever this creature attacks with another creature with
/// greater power, put a +1/+1 counter on this creature." Rural Recruit (1/1)
/// grows when it attacks beside a bigger creature, and not when alone.
#[test]
fn cr_702_149_training_grows_beside_a_bigger_attacker() {
    let mut g = two_player_game();
    let recruit = g.add_card_to_battlefield(0, catalog::rural_recruit()); // 1/1 trainer
    let big = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    g.clear_sickness(recruit);
    g.clear_sickness(big);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: recruit, target: AttackTarget::Player(1) },
        Attack { attacker: big, target: AttackTarget::Player(1) },
    ]))
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(recruit).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1),
        "training fired beside a bigger attacker",
    );
}

/// CR 702.149a — a lone training attacker (no bigger ally attacking) gets no
/// counter.
#[test]
fn cr_702_149_training_silent_when_alone() {
    let mut g = two_player_game();
    let recruit = g.add_card_to_battlefield(0, catalog::rural_recruit());
    g.clear_sickness(recruit);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: recruit,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(recruit).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        None,
        "no bigger ally → no training counter",
    );
}

// ── CR 702.110 — Exploit ─────────────────────────────────────────────────────

/// CR 702.110b/c — Exploit lets the creature be sacrificed on entry; its
/// "when this exploits a creature" payoff then resolves. Mindleech Ghoul makes
/// each opponent exile a card from hand.
#[test]
fn cr_702_110_exploit_payoff_fires_on_sacrifice() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // accept exploit
    let opp_hand = g.players[1].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::mindleech_ghoul());
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent exiled a card from hand");
}

// ── CR 702.147 — Decayed (attack-sacrifice half) ─────────────────────────────

/// CR 702.147a — "When it attacks, sacrifice it at end of combat." A decayed
/// Zombie that attacks is gone by the post-combat main phase.
#[test]
fn cr_702_147_decayed_attacker_sacrificed_at_end_of_combat() {
    let mut g = two_player_game();
    let proc = g.add_card_to_battlefield(0, catalog::ghoulish_procession());
    let trig = catalog::ghoulish_procession().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(proc, 0, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    let zombie = g.battlefield.iter().find(|c| c.is_token).map(|c| c.id).unwrap();
    g.clear_sickness(zombie);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: zombie,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(zombie).is_none(), "decayed attacker sacrificed at end of combat");
}

/// CR 706.4 — a result-gated roll trigger ("whenever you roll a 5 or higher")
/// fires off the `RolledDice` event's greatest result. Ground Pounder gains
/// trample on a 5+, and stays vanilla on a low roll.
#[test]
fn cr_706_4_die_result_trigger_grants_trample() {
    use crate::card::Keyword;
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let roll = |face: u8| {
        let mut g = two_player_game();
        let gp = g.add_card_to_battlefield(0, catalog::ground_pounder());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(face)]));
        let effect = catalog::ground_pounder().activated_abilities[0].effect.clone();
        let ctx = crate::game::effects::EffectContext::for_ability(gp, 0, None);
        let evs = g.resolve_effect(&effect, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        g.computed_permanent(gp).unwrap().keywords.contains(&Keyword::Trample)
    };
    assert!(roll(5), "rolling a 5 grants trample");
    assert!(!roll(3), "rolling a 3 does not");
}

/// CR 702.146e — a Disturb back face that's an Aura is exiled (not put into a
/// graveyard) when it leaves the battlefield, just like a creature back.
#[test]
fn cr_702_146e_disturb_aura_back_exiles_on_leave() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_graveyard(0, catalog::kindly_ancestor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastDisturb {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
    }).expect("disturb the Aura");
    drain_stack(&mut g);
    // Kill the host; the orphaned Aura would normally go to the graveyard.
    g.battlefield_find_mut(bear).unwrap().damage = 2;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == id), "Aura back exiled, not in graveyard");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != id), "not in graveyard");
}

// ── CR 111.10 — "When you create a token" triggers ───────────────────────────

/// CR 111.10 — `EventKind::TokenCreated` fires per token created. Voldaren
/// Bloodcaster (4 Blood already out) crosses its five-Blood transform threshold
/// when its fifth Blood is minted.
#[test]
fn cr_111_10_token_created_fires_per_token() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let caster = g.add_card_to_battlefield(0, catalog::voldaren_bloodcaster());
    for _ in 0..4 {
        g.add_token_to_battlefield(0, &crate::game::effects::blood_token());
    }
    let evs = g.resolve_effect(
        &crate::card::Effect::CreateToken {
            who: crate::effect::PlayerRef::You,
            count: crate::card::Value::Const(1),
            definition: crabomination_base::tokens::blood_token(),
        },
        &EffectContext::for_ability(crate::card::CardId(0), 0, None),
    ).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(caster).unwrap().definition.name, "Bloodbat Summoner");
}

/// CR 614.13 — a token-doubling replacement multiplies the count, and each
/// resulting token fires its own `TokenCreated` event (CR 111.10). With a
/// doubler out, Voldaren at 3 Blood mints one → two Blood enter → it reaches
/// five and transforms off the doubled token.
#[test]
fn cr_614_13_token_doubling_fires_per_doubled_token() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let caster = g.add_card_to_battlefield(0, catalog::voldaren_bloodcaster());
    g.add_card_to_battlefield(0, catalog::elspeth_storm_slayer()); // doubles tokens
    for _ in 0..3 {
        g.add_token_to_battlefield(0, &crate::game::effects::blood_token());
    }
    let evs = g.resolve_effect(
        &crate::card::Effect::CreateToken {
            who: crate::effect::PlayerRef::You,
            count: crate::card::Value::Const(1),
            definition: crabomination_base::tokens::blood_token(),
        },
        &EffectContext::for_ability(crate::card::CardId(0), 0, None),
    ).unwrap();
    // The single mint doubled to two tokens → five Blood total.
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Blood").count(),
        5,
        "doubler made two Blood from one mint"
    );
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(caster).unwrap().definition.name, "Bloodbat Summoner");
}

// ── CR 508.1a — attack-only restriction ──────────────────────────────────────

/// CR 508.1a — "can't attack unless you control [N matching]" restricts only
/// attacking, not blocking, when flagged attack-only (Lambholt Pacifist).
#[test]
fn cr_508_1a_attack_only_gate_allows_blocking() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let pacifist = g.add_card_to_battlefield(0, catalog::lambholt_pacifist());
    g.clear_sickness(pacifist);
    g.step = TurnStep::DeclareAttackers;
    assert!(!g.legal_attackers(0).contains(&pacifist), "no power-4 creature → can't attack");
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    assert!(g.blocker_can_block_attacker(pacifist, atk), "but it can still block");
}

// ── CR 509.1b — "can block only creatures with flying" restriction ────────────

/// Shacklegeist (and Wanderlight Spirit) can block only flyers: a ground
/// attacker can't be blocked by it, but a flyer can.
#[test]
fn cr_509_1b_can_block_only_flying_restriction() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let blk = g.add_card_to_battlefield(1, catalog::shacklegeist());
    let binst = g.battlefield_find(blk).unwrap();
    let bcomp = g.computed_permanent(blk).unwrap();
    assert!(
        !crate::game::can_block_attacker_computed(binst, &bcomp, &[], &[], 3),
        "ground attacker can't be blocked by a fly-only blocker"
    );
    assert!(
        crate::game::can_block_attacker_computed(binst, &bcomp, &[Keyword::Flying], &[], 3),
        "a flyer can be blocked"
    );
}

// ── CR 726 — Day and Night (spell-driven) ────────────────────────────────────

/// A spell with "It becomes night" sets the day/night state to night from
/// outside the upkeep transition.
#[test]
fn cr_726_spell_becomes_night() {
    use crate::game::effects::EffectContext;
    use crate::game::types::DayNight;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    assert_eq!(g.day_night, None, "starts neither day nor night");
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::into_the_night().effect, &ctx).unwrap();
    assert_eq!(g.day_night, Some(DayNight::Night), "spell forced night");
}
