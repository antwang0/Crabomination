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
//! token-doubling interaction (CR 614.13), attack-only "can't attack
//! unless you control N" restrictions (CR 508.1a), the Ring-bearer's
//! granted Legendary supertype (CR 701.54c / 205.4b), block-only combat
//! restrictions (CR 509.1c), one-sided "deals damage equal to its
//! power" effects (CR 701.12-style), Craft (CR 702.169), the Descend ability
//! word (CR 207.2c — permanent cards in the graveyard), losing on an
//! empty-library draw (CR 104.3c / 704.5c), characteristic-defining P/T that
//! recomputes live (CR 604.3 — Lhurgoyf), ownership independent of control
//! (CR 108.3 / 800.4a — Gruul Charm's "gain control of permanents you own"),
//! and Cascade granted to other spells (CR 702.85 — The First Sliver),
//! abilities-only mana spend restrictions (CR 605.1c — Omen Hawker),
//! controller-graveyard characteristic-defining P/T (CR 604.3 — Nethergoyf),
//! "can't block" as an illegal-blocker restriction (CR 509.1b), the Earthbend
//! lifecycle (CR 701.66a — becomes a 0/0 haste land creature, returns tapped on
//! death), and lethal-damage / 0-toughness SBAs measured against the *computed*
//! type so animated lands die like creatures (CR 704.5f/g), combat damage to a
//! planeswalker removing loyalty (CR 306.9 / 508.4), the Flying/Reach block
//! restriction (CR 702.4c / 702.9c), and Equip timing/control restrictions
//! (CR 702.6c/d) plus re-equip move semantics (CR 301.5c), last-known
//! information for a source sacrificed as a cost (CR 608.2h — Blazing Bomb),
//! Landcycling fetching a basic of the named type (CR 702.28e), and layer-7
//! P/T ordering where a dynamic anthem stacks with +1/+1 counters and
//! re-scales live to the board (CR 613.7 — Warrior of Light), Aura-granted
//! indestructible surviving lethal damage (CR 704.5g — Shielded by Faith), an
//! Opalescence-animated enchantment dying to lethal (CR 613), a
//! protection-from-creatures attacker being unblockable (CR 702.16e), reflexive
//! "when you do" payoffs gated on the action (CR 603.12 — Bloodcrazed
//! Socialite), loyalty abilities once-per-turn at sorcery speed (CR 606.3 —
//! Nissa), and deathtouch making any damage lethal (CR 702.2b), Menace as a
//! two-blocker requirement (CR 702.111 — Insatiable Skittermaw), lifelink on
//! noncombat ability damage (CR 702.15 — Merciless Enforcers), and
//! target-conditional cost reduction (CR 601.2f — Grow Extra Arms costs {1}
//! less targeting a Spider), last-known controller of a died creature
//! (CR 603.10 — Furious Forebear's graveyard trigger reads LKI so it fires
//! only for your creatures), variable "Pay X life" activation costs
//! (CR 107.16 — Krumar Initiate), and Mobilize's tapped-attacking transient
//! tokens (CR 702.169 — Zurgo's Vanguard), Undying returning only without a
//! +1/+1 counter (CR 702.93), Toxic N poison on combat damage (CR 702.180),
//! and Wither dealing creature damage as -1/-1 counters but player damage as
//! life (CR 702.71), Harmonize tapping a creature to reduce the cost by its
//! power (CR 702.180b), "until end of turn" grants ending at cleanup (CR 514.2
//! — a granted harmonize), and a one-sided damage doubler sparing the
//! controller's own side (CR 614.5), a permanent Gift given as it enters
//! (CR 702.165 — Scrapshooter firing Jolly Gerbils), conditional self-pump
//! stacking on +1/+1 counters in layer 7 (CR 613.7c — Aven Heartstabber), and
//! {X} in a card's cost counting as 0 outside the stack (CR 202.3b).

use crabomination::catalog;
use crabomination::card::CounterType;
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::mana::Color;
use crabomination::game::two_player_game;
use crabomination::game::*;

// ── CR 701.35 — Detain ────────────────────────────────────────────────────────

#[test]
fn cr_701_35_detain_stops_attack_block_and_activation_until_detainers_next_turn() {
    let mut g = two_player_game();
    // Opponent (seat 1) controls a creature that we'll detain via Lyev Skyknight.
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(victim);
    // Cast Lyev Skyknight (seat 0) and detain the bear on ETB.
    let lyev = g.add_card_to_hand(0, catalog::lyev_skyknight());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crabomination::game::cast_at(&mut g, lyev, Target::Permanent(victim));
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
fn fateseal_two() -> crabomination::card::CardDefinition {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::effect::{Effect, PlayerRef, Value};
    CardDefinition {
        name: "Test Fateseal 2",
        cost: crabomination::mana::cost(&[crabomination::mana::generic(1), crabomination::mana::u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Fateseal { who: PlayerRef::EachOpponent, amount: Value::Const(2) },
        ..Default::default()
    }
}

#[test]
fn cr_701_29_fateseal_bottoms_chosen_card_of_opponent_library() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Opponent's top two library cards.
    let top = g.add_card_to_library(1, catalog::island());
    let _second = g.add_card_to_library(1, catalog::forest());
    let before_len = g.players[1].library.len();
    let spell = g.add_card_to_hand(0, fateseal_two());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Put the opponent's top card (`top`) on the bottom.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![top])]));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crabomination::game::cast(&mut g, spell);
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
    use crabomination::card::CounterType;
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
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crabomination::game::cast_at(&mut g, murder, Target::Permanent(opp));
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
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
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
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
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
    use crabomination::effect::{Effect, PlayerRef};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    for p in 0..2 {
        let id = g.next_id();
        g.players[p].graveyard.push(crabomination::card::CardInstance::new(
            id,
            catalog::lightning_bolt(),
            p,
        ));
    }
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(&Effect::ExilePlayerGraveyard { who: PlayerRef::EachPlayer, filter: None }, &ctx)
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
    kws: &[crabomination::card::Keyword],
) -> crabomination::card::CardDefinition {
    use crabomination::card::{CardDefinition, CardType};
    CardDefinition {
        name,
        cost: crabomination::mana::cost(&[crabomination::mana::g()]),
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
    use crabomination::card::Keyword;
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::effects::EffectContext;
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
    assert_eq!(c.counter_count(crabomination::card::CounterType::MinusOneMinusOne), 2);
    assert_eq!(c.damage, 0, "wither damage is not marked damage");
}

/// A nonzero non-combat ping from a deathtouch source destroys the damaged
/// creature at the next SBA check (CR 702.2c).
#[test]
fn cr_702_2c_noncombat_deathtouch_ping_destroys() {
    use crabomination::card::Keyword;
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::effects::EffectContext;
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
    use crabomination::card::Keyword;
    use crabomination::effect::{Effect, Selector};
    use crabomination::game::effects::EffectContext;
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
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::leyline_binding(), 0);
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
    use crabomination::card::CounterType;
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
    use crabomination::card::CounterType;
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
    use crabomination::card::CounterType;
    use crabomination::game::types::Attack;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let jitte = g.add_card_to_battlefield(0, catalog::umezawas_jitte());
    g.battlefield_find_mut(jitte).unwrap().attached_to = Some(attacker);
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(1) }];
    g.set_block_map([(blocker, attacker)]);
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
    use crabomination::card::CounterType;
    use crabomination::game::types::Attack;
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
    use crabomination::game::types::Attack;
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
    use crabomination::game::types::Attack;
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Attack;
    let mut g = two_player_game();
    // Say "yes" to the encode prompt and the later free-copy prompt.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let slice = g.add_card_to_hand(0, catalog::shadow_slice());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    crabomination::game::cast_at(&mut g, slice, Target::Player(1));
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
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
    crabomination::game::cast_at(&mut g, bolt, Target::Player(0));
    assert_eq!(g.players[0].life, 20, "player untouched");
    assert_eq!(g.battlefield_find(giant).unwrap().damage, 3, "giant soaked the bolt");
    // Bolt at the bear: also redirected.
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
    crabomination::game::cast_at(&mut g, bolt2, Target::Permanent(bear));
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
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
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
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::CastFlashback {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no card to discard → can't jump-start");
}

// ── CR 724 — Ending the Turn ──────────────────────────────────────────────────

/// Sundial of the Infinite ends the turn: a spell still on the stack is
/// exiled (not resolved), combat state clears, and play skips to cleanup.
#[test]
fn cr_724_sundial_exiles_the_stack_and_skips_to_cleanup() {
    let mut g = two_player_game();
    let sundial = g.add_card_to_battlefield(0, catalog::sundial_of_the_infinite());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast bolt");
    g.perform_action(GameAction::ActivateAbility {
        card_id: sundial, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Sundial");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "bolt exiled off the stack (728.1a)");
    assert_eq!(g.players[1].life, 20, "bolt never resolved");
    // CR 724.1d + 514.3 — the turn skips to cleanup, which grants no
    // priority and ends the turn: play resumes in the opponent's upkeep.
    assert_eq!(g.active_player_idx, 1, "turn ended (728.1d)");
    assert_eq!(g.step, TurnStep::Upkeep, "no cleanup priority (514.3)");
}

/// Sundial's "activate only during your turn" gate rejects an off-turn use.
#[test]
fn cr_724_sundial_rejects_activation_on_opponents_turn() {
    let mut g = two_player_game();
    let sundial = g.add_card_to_battlefield(1, catalog::sundial_of_the_infinite());
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: sundial, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "only during your turn");
}

/// Day's Undoing wheels both players (hand + graveyard shuffled into
/// library, draw seven) and, on the caster's turn, ends the turn — the
/// sorcery itself is exiled with the stack (728.1a).
#[test]
fn cr_724_days_undoing_wheels_then_ends_the_turn() {
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
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let tender = g.add_card_to_battlefield(0, catalog::burrenton_forge_tender());
    g.clear_sickness(tender);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(mine)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast bolt at the bear");
    // In response: sacrifice the Forge-Tender choosing the bolt as source.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bolt])]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tender, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let tender = g.add_card_to_battlefield(1, catalog::burrenton_forge_tender());
    g.clear_sickness(tender);
    let raider = g.add_card_to_battlefield(0, catalog::ball_lightning());
    g.clear_sickness(raider);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![raider])]));
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tender, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
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
        k.counter_count(crabomination::card::CounterType::PlusOnePlusOne),
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
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
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
    use crabomination::effect::{Duration, Selector, Value};
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
    use crabomination::effect::{Duration, Selector, Value};
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
    use crabomination::effect::{Duration, Selector, Value};
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
    g.move_card_to(bear, &crabomination::effect::ZoneDest::Hand(crabomination::effect::PlayerRef::You), &ctx, &mut events);
    let hand_pos = g.players[0].hand.iter().position(|c| c.id == bear).unwrap();
    let card = g.players[0].hand.remove(hand_pos);
    g.battlefield.push(card);
    let computed = g.computed_permanent(bear).expect("computed");
    assert_eq!((computed.power, computed.toughness), (2, 2), "pump ended on zone change");
}

// ── CR 510.1c — a blocked attacker remains blocked ────────────────────────────

/// Test-only fixture: a 3/3 double striker, optionally with trample.
fn double_striker(trample: bool) -> crabomination::card::CardDefinition {
    use crabomination::card::{CardDefinition, CardType, Keyword};
    let mut keywords = vec![Keyword::DoubleStrike];
    if trample {
        keywords.push(Keyword::Trample);
    }
    CardDefinition {
        name: "Test Double Striker",
        cost: crabomination::mana::cost(&[crabomination::mana::generic(2), crabomination::mana::r()]),
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

/// CR 506.4c — a creature attacking a planeswalker stays in combat when that
/// planeswalker leaves, but is "attacking no player"; if unblocked it deals no
/// combat damage (and nothing is redirected to the defending player).
#[test]
fn cr_506_4c_attacker_whose_planeswalker_left_deals_no_damage() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(attacker);
    let pw = g.add_card_to_battlefield(1, catalog::teferi_time_raveler());
    let life_before = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Planeswalker(pw),
    }]))
    .unwrap();
    // The attacked planeswalker leaves the battlefield before combat damage.
    g.battlefield.retain(|c| c.id != pw);
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert_eq!(
        g.players[1].life, life_before,
        "an unblocked attacker whose planeswalker left combat deals no damage to the player",
    );
}

/// CR 702.52 — Dredge: if you would draw a card, you may instead mill N and
/// return the dredge card from your graveyard to your hand (no card drawn).
#[test]
fn cr_702_52_dredge_mills_and_returns_instead_of_drawing() {
    let mut g = two_player_game();
    let brownscale = g.add_card_to_graveyard(0, catalog::golgari_brownscale()); // Dredge 2
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let mut events = vec![];
    g.draw_one(0, &mut events);
    assert!(
        g.players[0].hand.iter().any(|c| c.id == brownscale),
        "the dredge card returns to hand instead of a draw",
    );
    assert_eq!(g.players[0].library.len(), 1, "2 of the 3 library cards were milled");
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.id == brownscale),
        "the dredge card left the graveyard",
    );
}

// ── CR 601.2c — cast-time target filters enforced for every targeted effect ──

/// Detain was one of ~20 targeted effects whose filter wasn't surfaced by
/// `target_filter_for_slot`, letting a client submit any target (the caster's
/// own land). The filter must reject illegal targets at cast time.
#[test]
fn cr_601_2c_detain_filter_rejects_own_land() {
    use crabomination::card::{CardDefinition, CardType, SelectionRequirement};
    use crabomination::effect::{Effect, Selector};
    fn detain_spell() -> CardDefinition {
        CardDefinition {
            name: "Test Detain",
            cost: crabomination::mana::cost(&[crabomination::mana::u()]),
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
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
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
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    let bear_a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear_b = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let source = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Push a trigger aimed at bear_a, then remove bear_a before resolution.
    g.stack.push(crabomination::game::types::StackItem::Trigger {
        source,
        controller: 0,
        effect: Box::new(Effect::DealDamage {
            to: Selector::TargetFiltered {
                slot: 0,
                filter: crabomination::card::SelectionRequirement::Creature,
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
        trigger_player: None,
        intervening_if: None,
        additional_targets: Vec::new(),
        mana_spent_by_color: Vec::new(),
        activated: false,
    });
    let mut events = Vec::new();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.move_card_to(bear_a, &crabomination::effect::ZoneDest::Hand(crabomination::effect::PlayerRef::Seat(1)), &ctx, &mut events);
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
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
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
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
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
    use crabomination::effect::{Duration, Effect, Selector};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let eff = Effect::GainControl {
        what: Selector::TargetFiltered {
            slot: 0,
            filter: crabomination::card::SelectionRequirement::Creature,
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
    use crabomination::game::effects::blood_token;
    let mut g = two_player_game();
    let blood = g.add_token_to_battlefield(0, &blood_token());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].hand.clear();
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: blood, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        })
        .is_err(),
        "empty hand can't pay the Blood discard cost"
    );
}

/// CR 702.47a — Soulshift returns a Spirit from YOUR graveyard, never an
/// opponent's.
#[test]
fn cr_702_47a_soulshift_only_fetches_own_graveyard() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
    use crabomination::effect::{Duration, Effect, Selector, Value};
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
    assert_eq!(g.attackers_blocked_by(land), [attacker]);
}

// ── CR 510.1c/d — marked damage + full assignment ─────────────────────────────

/// A double-strike trampler only needs the blocker's REMAINING toughness as
/// lethal in the regular step — the rest tramples over (CR 510.1c).
#[test]
fn cr_510_1c_marked_damage_counts_toward_lethal() {
    use crabomination::card::{CardDefinition, CardType, Keyword};
    fn ds_trampler() -> CardDefinition {
        CardDefinition {
            name: "Test DS Trampler",
            cost: crabomination::mana::cost(&[crabomination::mana::generic(3), crabomination::mana::r()]),
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
            cost: crabomination::mana::cost(&[crabomination::mana::generic(2)]),
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
    use crabomination::card::{CardDefinition, CardType, Keyword};
    fn indestructible_2_2() -> CardDefinition {
        CardDefinition {
            name: "Test Indestructible Bear",
            cost: crabomination::mana::cost(&[crabomination::mana::generic(2)]),
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
    use crabomination::effect::{Effect, Selector, Value};
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
    use crabomination::card::Keyword;
    use crabomination::effect::{Effect, Selector};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().keyword_counters.insert(Keyword::Flying, 1);
    g.battlefield_find_mut(bear)
        .unwrap()
        .add_counters(crabomination::card::CounterType::PlusOnePlusOne, 2);
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
    use crabomination::card::Keyword;
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
    assert_eq!(a.counter_count(crabomination::card::CounterType::MinusOneMinusOne), 2,
        "only the infect blocker's 2 power lands as counters");
    assert_eq!(a.damage, 3, "the vanilla blocker's 3 power is marked damage");
}

/// Source-scoped damage scaling (Torbran) applies per strike-back event,
/// not once to the summed total (CR 614.5).
#[test]
fn cr_702_90_strike_back_scaling_is_per_source() {
    use crabomination::card::{CardDefinition, CardType};
    let red_body = |name: &'static str, p: i32, t: i32| CardDefinition {
        name,
        cost: crabomination::mana::cost(&[crabomination::mana::r()]),
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
    use crabomination::card::{CardDefinition, CardType, Keyword, SelectionRequirement, StaticAbility};
    use crabomination::effect::{Duration, Effect, Selector, StaticEffect};
    use crabomination::game::effects::EffectContext;
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
    use crabomination::card::{ArtifactSubtype, CardDefinition, CardType, EquipBonus, Keyword, Subtypes};
    use crabomination::effect::{Duration, Effect, Selector};
    use crabomination::game::effects::EffectContext;
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
    use crabomination::card::{CardDefinition, CardType, TriggeredAbility};
    use crabomination::effect::{Effect, EventKind, EventScope, EventSpec, Selector, Value};
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
                for (k, v) in m {
                    // A `CreateToken`'s embedded `TokenDefinition` ("definition"/
                    // "token") carries the *token's own* ability targets, chosen
                    // when that ability is activated — not cast-time targets of
                    // the spell. Don't count them (e.g. a Map token's "target
                    // creature you control explores" inside Fanatical Offering).
                    if k == "definition" || k == "token" {
                        continue;
                    }
                    // CR 702.172 (Spree) / FIN Tiered — modes each use their own
                    // internal slot 0; the resolver remaps targets per chosen
                    // mode, so their filters aren't surfaced through the single-
                    // `mode` cast path (which can't express which modes were
                    // chosen). Targets are validated per-mode at resolution.
                    if k == "Spree" || k == "Tiered" {
                        continue;
                    }
                    // CR 603.7 — a `Reflexive` "when you do" payoff chooses its
                    // targets at resolution (after the gating cost), not at
                    // cast time, so its slots are intentionally opaque to the
                    // cast-time surfacing walk (Glacial Dragonhunt's bolt).
                    if k == "Reflexive" {
                        continue;
                    }
                    // A `GrantTriggeredAbility`'s granted `trigger` chooses its
                    // own targets when that trigger later goes on the stack, not
                    // at the granting spell's cast (Showstopper's death ping).
                    if k == "trigger" {
                        continue;
                    }
                    // Likewise a `GainActivatedAbility`'s granted `ability`:
                    // its targets are chosen when the grantee activates it
                    // (Lightning Volley's tap-ping).
                    if k == "ability" {
                        continue;
                    }
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
    for f in crabomination::catalog::all_known_factories() {
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
    use crabomination::card::{CardDefinition, CardType, TriggeredAbility};
    use crabomination::effect::{Effect, EventKind, EventScope, EventSpec, Selector, Value};
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
        cost: crabomination::mana::cost(&[crabomination::mana::phyrexian(crabomination::mana::Color::Black)]),
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
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
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
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
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
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
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
    g.resolve_effect(&Effect::Destroy { what: crabomination::card::Selector::EachPermanent(
        crabomination::card::SelectionRequirement::Creature) }, &ctx).unwrap();
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
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("opponent bolt");
    let fork = g.add_card_to_hand(0, catalog::reverberate());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: fork, target: Some(Target::Permanent(bolt)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Reverberate the bolt");
    // Resolve Reverberate only — the copy is now on the stack.
    g.resolve_top_of_stack().expect("fork resolves");
    let copy_id = match g.stack.last() {
        Some(crabomination::game::StackItem::Spell { card, .. }) if card.is_token => card.id,
        other => panic!("expected the copy on top, got {other:?}"),
    };
    let lapse = g.add_card_to_hand(0, catalog::memory_lapse());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
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
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
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
        card_id: mountain, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
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
        card_id: stone, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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
    use crabomination::effect::{Selector, Value};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lymph_sliver());
    let subject = g.add_card_to_battlefield(0, catalog::crystalline_sliver()); // 2/2
    // Hit the 2/2 Sliver for 2 → absorb 1 → 1 marked damage, survives.
    let ctx = crabomination::game::effects::EffectContext::for_spell(
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
    g.set_block_map([(sliver, bear)]);
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
    use crabomination::card::{EnchantmentSubtype, EquipBonus, TokenDefinition};
    use crabomination::effect::Selector;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let role = TokenDefinition {
        name: "Wicked".into(),
        card_types: vec![crabomination::card::CardType::Enchantment],
        subtypes: crabomination::card::Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Role],
            ..Default::default()
        },
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        ..Default::default()
    };
    let ctx = crabomination::game::effects::EffectContext::for_ability(bear, 0, None);
    let mint = Effect::CreateTokenAttachedTo { target: Selector::This, definition: role };
    g.resolve_effect(&mint, &ctx).unwrap();
    let first: Vec<CardId> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Wicked").map(|c| c.id).collect();
    assert_eq!(first.len(), 1);
    g.resolve_effect(&mint, &ctx).unwrap();
    g.check_state_based_actions();
    let roles: Vec<&crabomination::card::CardInstance> = g.battlefield.iter()
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
    use crabomination::card::{EnchantmentSubtype, TokenDefinition};
    use crabomination::effect::Selector;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let role = TokenDefinition {
        name: "Cursed".into(),
        card_types: vec![crabomination::card::CardType::Enchantment],
        subtypes: crabomination::card::Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Role],
            ..Default::default()
        },
        ..Default::default()
    };
    let mint = Effect::CreateTokenAttachedTo { target: Selector::This, definition: role };
    g.resolve_effect(&mint, &crabomination::game::effects::EffectContext::for_ability(bear, 0, None)).unwrap();
    g.resolve_effect(&mint, &crabomination::game::effects::EffectContext::for_ability(bear, 1, None)).unwrap();
    g.check_state_based_actions();
    let n = g.battlefield.iter().filter(|c| c.definition.name == "Cursed").count();
    assert_eq!(n, 2, "one Role per controller may stay");
}

/// Nylea, Keen-Eyed's nonland miss offers "put it into your graveyard";
/// accepting bins the reveal, declining leaves it on top.
#[test]
fn nylea_reveal_miss_may_go_to_graveyard() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let top = g.add_card_to_library(0, catalog::lightning_bolt()); // not a creature
    let nylea = g.add_card_to_battlefield(0, catalog::nylea_keen_eyed());
    g.clear_sickness(nylea);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: nylea, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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
    assert!(g.computed_permanent(mine).unwrap().keywords.contains(&crabomination::card::Keyword::Haste));
    assert!(g.computed_permanent(theirs).unwrap().keywords.contains(&crabomination::card::Keyword::Haste));
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
        crabomination::decision::Decision::ChooseCards { candidates, min, max, .. } => {
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
    assert!(matches!(pd.decision, crabomination::decision::Decision::ChooseCards { .. }));
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
        card_id: cam, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("first exhaust activation");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cam).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "got a +1/+1 counter");
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Thopter"),
        "created a Thopter token");

    // Same turn → rejected.
    g.players[0].mana_pool.add_colorless(3);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: cam, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "exhaust ability can't be activated twice");

    // Turn cleanup clears once-per-turn state, but exhaust persists (per game).
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == cam) {
        c.clear_end_of_turn_effects();
    }
    g.players[0].mana_pool.add_colorless(3);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: cam, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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
        card_id: h, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("exhaust");
    drain_stack(&mut g);
    let c = g.battlefield_find(h).unwrap();
    assert_eq!((c.power(), c.toughness()), (7, 7), "4/4 + three +1/+1 = 7/7");
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(6);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: h, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "exhaust can't repeat");
}

/// Pacesetter Paragon's exhaust adds a +1/+1 counter and grants double strike
/// until end of turn.
#[test]
fn cr_702_177_pacesetter_paragon_exhaust_grants_double_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::pacesetter_paragon());
    g.clear_sickness(p);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: p, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let gg = g.add_card_to_battlefield(0, catalog::greenbelt_guardian());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(gg);
    // Non-exhaust ability (index 0): grant trample to the bear, twice.
    for _ in 0..2 {
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: gg, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("repeatable trample grant");
        drain_stack(&mut g);
    }
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample));
    // Exhaust ability (index 1): +3/+3 once.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gg, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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
        card_id: p, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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
        card_id: k, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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
        card_id: s, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("exhaust");
    drain_stack(&mut g);
    let c = g.battlefield_find(s).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 5));
}

/// Mai, Jaded Edge's exhaust puts a double-strike counter on her.
#[test]
fn cr_702_177_mai_jaded_edge_double_strike_counter() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let m = g.add_card_to_battlefield(0, catalog::mai_jaded_edge());
    g.clear_sickness(m);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: m, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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
        card_id: s, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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
        card_id: m, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: Some(2), mode: None,
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
    let ctx = crabomination::game::effects::EffectContext::for_trigger(hok, 0, None, 0);
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
    let assert_block = |def: crabomination::card::CardDefinition, expect: bool, why: &str| {
        let mut g = two_player_game();
        let blk = g.add_card_to_battlefield(1, def);
        let inst = g.battlefield_find(blk).unwrap().clone();
        let cp = g.computed_permanent(blk).unwrap();
        assert_eq!(
            crabomination::game::can_block_attacker_computed(&inst, &cp, attacker_kws, &[], 2),
            expect, "{why}"
        );
    };
    assert_block(catalog::grizzly_bears(), false, "green creature can't block Fear");
    assert_block(catalog::ornithopter(), true, "artifact creature can block Fear");
    assert_block(catalog::nezumi_cutthroat(), true, "black creature can block Fear");
}

/// CR 702.13 — Intimidate: an attacker with Intimidate can be blocked only by
/// artifact creatures and creatures sharing a color with it.
#[test]
fn cr_702_13_intimidate_blockable_only_by_artifact_or_shared_color() {
    use crabomination::card::Keyword;
    let attacker_kws = [Keyword::Intimidate];
    let attacker_colors = [Color::Red];
    let assert_block = |def: crabomination::card::CardDefinition, expect: bool, why: &str| {
        let mut g = two_player_game();
        let blk = g.add_card_to_battlefield(1, def);
        let inst = g.battlefield_find(blk).unwrap().clone();
        let cp = g.computed_permanent(blk).unwrap();
        assert_eq!(
            crabomination::game::can_block_attacker_computed(&inst, &cp, &attacker_kws, &attacker_colors, 2),
            expect, "{why}"
        );
    };
    assert_block(catalog::grizzly_bears(), false, "green shares no color with a red attacker");
    assert_block(catalog::goblin_guide(), true, "red creature shares color");
    assert_block(catalog::ornithopter(), true, "artifact creature can block Intimidate");
}

/// CR 702.72a — Skulk: an attacker can't be blocked by a creature with greater
/// power (computed, so anthem-pumped power counts).
#[test]
fn cr_702_72_skulk_blocked_only_by_equal_or_lesser_power() {
    use crabomination::card::{CardDefinition, CardType, Keyword};
    let vanilla = |power: i32, toughness: i32| CardDefinition {
        name: "Vanilla",
        card_types: vec![CardType::Creature],
        power,
        toughness,
        ..Default::default()
    };
    let attacker_kws = [Keyword::Skulk];
    let assert_block = |def: CardDefinition, expect: bool, why: &str| {
        let mut g = two_player_game();
        let blk = g.add_card_to_battlefield(1, def);
        let inst = g.battlefield_find(blk).unwrap().clone();
        let cp = g.computed_permanent(blk).unwrap();
        assert_eq!(
            crabomination::game::can_block_attacker_computed(&inst, &cp, &attacker_kws, &[], 2),
            expect, "{why}"
        );
    };
    assert_block(vanilla(1, 1), true, "lesser power may block a Skulk 2-power attacker");
    assert_block(vanilla(2, 2), true, "equal power may block");
    assert_block(vanilla(3, 3), false, "greater power can't block");
}

/// CR 509.1b — a fixed-threshold block restriction: "can't be blocked by
/// creatures with power 3 or greater" (Squeak By keyword).
#[test]
fn cr_509_1b_cant_be_blocked_by_power_at_least() {
    use crabomination::card::{CardDefinition, CardType, Keyword};
    let vanilla = |power: i32| CardDefinition {
        name: "Vanilla",
        card_types: vec![CardType::Creature],
        power,
        toughness: power.max(1),
        ..Default::default()
    };
    let attacker_kws = [Keyword::CantBeBlockedByPowerAtLeast(3)];
    let assert_block = |def: CardDefinition, expect: bool, why: &str| {
        let mut g = two_player_game();
        let blk = g.add_card_to_battlefield(1, def);
        let inst = g.battlefield_find(blk).unwrap().clone();
        let cp = g.computed_permanent(blk).unwrap();
        assert_eq!(
            crabomination::game::can_block_attacker_computed(&inst, &cp, &attacker_kws, &[], 5),
            expect, "{why}"
        );
    };
    assert_block(vanilla(2), true, "power 2 may block");
    assert_block(vanilla(3), false, "power 3 (≥ threshold) can't block");
    assert_block(vanilla(4), false, "power 4 can't block");
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
    use crabomination::card::CounterType;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
        additional_targets: Vec::new(), x_value: None, mode: None,
    });
    assert!(err.is_err(), "a red source's ability can't target protection from red");
    // A non-protected creature is a fine target.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None, mode: None,
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
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let labyrinth = g.add_card_to_battlefield(0, catalog::labyrinth_of_skophos());
    g.clear_sickness(labyrinth);
    g.set_attacking(vec![Attack { attacker, target: AttackTarget::Player(0) }]);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: labyrinth, ability_index: 1, target: Some(Target::Permanent(attacker)),
        additional_targets: Vec::new(), x_value: None, mode: None,
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

/// CR 601.2f — Gran-Gran's Lesson-gated player-wide reduction
/// (`StaticEffect::CostReductionWhile`) discounts only generic mana: with 3+
/// Lessons in the graveyard the reduction is active (cost_reduction = 1), yet a
/// {U} Lesson still can't be cast for free — the colored pip survives the clamp.
#[test]
fn cr_601_2f_gran_gran_lesson_discount_is_generic_only() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gran_gran());
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::yip_yip()); } // 3 Lessons → reduction on
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // The reduction is live for a noncreature Lesson…
    let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::boomerang_basics(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 1, "reduction active");

    // …but it can't pay the {U}: casting Boomerang Basics ({U}) with no mana fails.
    let boomerang = g.add_card_to_hand(0, catalog::boomerang_basics());
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: boomerang, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "discount can't make a colored-only spell free");
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
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Color(Color::Green),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: dork, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(m)),
        crabomination::decision::DecisionAnswer::Search(None),
        crabomination::decision::DecisionAnswer::Search(None),
        crabomination::decision::DecisionAnswer::Search(None),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(m).is_some(), "Minotaur tutored to battlefield");
    assert!(g.players[0].library.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the non-Minotaur stays in the library");
}

// ── CR 701.19c — a searched library is shuffled, found or not ─────────────────
/// Declining every pick still counts as searching, so the library must be
/// shuffled afterward (with 30 cards, an unchanged order is a ~1/30! fluke).
#[test]
fn cr_701_19c_search_shuffles_library_even_on_decline() {
    let mut g = two_player_game();
    for _ in 0..30 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let before: Vec<_> = g.players[0].library.iter().map(|c| c.id).collect();
    let spell = g.add_card_to_hand(0, catalog::deathbellow_war_cry());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add_colorless(5);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(None),
        crabomination::decision::DecisionAnswer::Search(None),
        crabomination::decision::DecisionAnswer::Search(None),
        crabomination::decision::DecisionAnswer::Search(None),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let after: Vec<_> = g.players[0].library.iter().map(|c| c.id).collect();
    let mut before_sorted = before.clone();
    let mut after_sorted = after.clone();
    before_sorted.sort();
    after_sorted.sort();
    assert_eq!(before_sorted, after_sorted, "nothing was taken — same cards remain");
    assert_ne!(before, after, "searching shuffles the library (CR 701.19c)");
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
        !g.computed_permanent(heliod).unwrap().card_types.contains(&crabomination::card::CardType::Creature),
        "devotion 4 < 5 — Heliod isn't a creature"
    );
    g.add_card_to_battlefield(0, catalog::altar_of_the_pantheon()); // +1 → 5
    assert!(
        g.computed_permanent(heliod).unwrap().card_types.contains(&crabomination::card::CardType::Creature),
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(1)])); // d3=1 → chosen 2
    let haktos = g.add_card_to_battlefield(0, catalog::haktos_the_unscarred());
    let eff = catalog::haktos_the_unscarred().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(haktos, 0, None, 0);
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(1)])); // d3=1 → chosen 2
    let haktos = g.add_card_to_battlefield(0, catalog::haktos_the_unscarred());
    let eff = catalog::haktos_the_unscarred().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(haktos, 0, None, 0);
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
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Deathtouch), "abilities/types kept");
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
    g.players[0].library.push(crabomination::card::CardInstance::new(nid, catalog::lightning_bolt(), 0));
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
    assert!(!crabomination::game::can_block_attacker_computed(
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
    assert!(!crabomination::game::can_block_attacker_computed(
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
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(
        [crabomination::decision::DecisionAnswer::Search(Some(small))],
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
    crabomination::game::cast_at(&mut g, swords, Target::Permanent(knight));
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
        crabomination::game::StackItem::Spell { card, .. } => *card,
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
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(
        [crabomination::decision::DecisionAnswer::Bool(true)],
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
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(
        [crabomination::decision::DecisionAnswer::Search(Some(forest))],
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
        crabomination::game::StackItem::Spell { card, .. } if card.id == bolt)),
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
    crabomination::game::cast_at(&mut g, re, Target::Permanent(tapped));
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
    use crabomination::card::ArtifactSubtype;
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
    crabomination::game::cast_at(&mut g, unburden, Target::Player(0));
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
    use crabomination::effect::{Effect, PlayerRef, Selector, Value};
    let mut g = two_player_game();
    g.players[1].life = 20;
    let choice = Effect::VillainousChoice {
        who: Selector::Player(PlayerRef::EachOpponent),
        option_a: Box::new(Effect::LoseLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(6) }),
        option_b: Box::new(Effect::LoseLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(2) }),
    };
    let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    g.resolve_effect(&choice, &ctx).unwrap();
    assert_eq!(g.players[1].life, 18, "opponent took the 2-life option");
}

/// CR 701.55b — an impossible option may be chosen and does nothing, so the
/// chooser dodges harm (no creature to sacrifice → sacrifice branch is free).
#[test]
fn cr_701_55_villainous_choice_takes_impossible_option_to_dodge() {
    use crabomination::card::SelectionRequirement;
    use crabomination::effect::{Effect, PlayerRef, Selector, Value};
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
    let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    g.resolve_effect(&choice, &ctx).unwrap();
    assert_eq!(g.players[1].life, 20, "dodged via the impossible sacrifice option");
}

// ── CR 614.16 — counter-placement replacement ordering ───────────────────────

/// CR 614.16 / 616.1 — Hardened Scales (additive +1) and a Doubling-Counters
/// permanent both replace a +1/+1 placement; the additive applies before the
/// doubling: a 1-counter placement becomes (1+1)*2 = 4.
#[test]
fn cr_614_16_additive_then_doubling_counter_replacement() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::hardened_scales());
    g.add_card_to_battlefield(0, catalog::witherbloom_pestseed()); // DoubleCounters
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)),
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
    use crabomination::effect::{Effect, PlayerRef};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::durkwood_baloth()); // Suspend 5—{G}
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::Suspend { card_id: id }).expect("suspend");
    let before = g.exile.iter().find(|c| c.id == id).unwrap().counter_count(CounterType::Time);
    assert_eq!(before, 5);
    let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::{Effect, Value, Selector};
    use crabomination::card::SelectionRequirement;
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
    let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
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
    let ctx = crabomination::game::effects::EffectContext::for_trigger(crabomination::card::CardId(99), 0, None, 0);
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
    let ctx = crabomination::game::effects::EffectContext::for_trigger(proc, 1, None, 0);
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
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::millicent_restless_revenant(), 0);
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
    let ctx = crabomination::game::effects::EffectContext::for_trigger(proc, 0, None, 0);
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
    use crabomination::card::Keyword;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let roll = |face: u8| {
        let mut g = two_player_game();
        let gp = g.add_card_to_battlefield(0, catalog::ground_pounder());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(face)]));
        let effect = catalog::ground_pounder().activated_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_ability(gp, 0, None);
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
        g.add_token_to_battlefield(0, &crabomination::game::effects::blood_token());
    }
    let evs = g.resolve_effect(
        &crabomination::card::Effect::CreateToken {
            who: crabomination::effect::PlayerRef::You,
            count: crabomination::card::Value::Const(1),
            definition: crabomination_base::tokens::blood_token(),
        },
        &EffectContext::for_ability(crabomination::card::CardId(0), 0, None),
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
        g.add_token_to_battlefield(0, &crabomination::game::effects::blood_token());
    }
    let evs = g.resolve_effect(
        &crabomination::card::Effect::CreateToken {
            who: crabomination::effect::PlayerRef::You,
            count: crabomination::card::Value::Const(1),
            definition: crabomination_base::tokens::blood_token(),
        },
        &EffectContext::for_ability(crabomination::card::CardId(0), 0, None),
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
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let blk = g.add_card_to_battlefield(1, catalog::shacklegeist());
    let binst = g.battlefield_find(blk).unwrap();
    let bcomp = g.computed_permanent(blk).unwrap();
    assert!(
        !crabomination::game::can_block_attacker_computed(binst, &bcomp, &[], &[], 3),
        "ground attacker can't be blocked by a fly-only blocker"
    );
    assert!(
        crabomination::game::can_block_attacker_computed(binst, &bcomp, &[Keyword::Flying], &[], 3),
        "a flyer can be blocked"
    );
}

// ── CR 731 — Day and Night (spell-driven) ────────────────────────────────────

/// A spell with "It becomes night" sets the day/night state to night from
/// outside the upkeep transition.
#[test]
fn cr_731_spell_becomes_night() {
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::DayNight;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    assert_eq!(g.day_night, None, "starts neither day nor night");
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::into_the_night().effect, &ctx).unwrap();
    assert_eq!(g.day_night, Some(DayNight::Night), "spell forced night");
}

/// CR 502.3 — Winter Orb prevents lands from untapping during their
/// controllers' untap steps (the `PreventUntap` machinery, exercised through a
/// real card). Non-land permanents are unaffected.
#[test]
fn cr_502_3_winter_orb_prevents_land_untap() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::winter_orb());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(land).unwrap().tapped, "land stays tapped under Winter Orb");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "non-land untaps");
}

/// CR 614.5 — noncombat damage replacements stack: Solphim's noncombat-only
/// doubler combines with Furnace of Rath's global doubler (3 → 12), while
/// combat damage only sees Furnace (Solphim is noncombat-only).
#[test]
fn cr_614_5_solphim_stacks_with_furnace_on_noncombat_only() {
    // Noncombat: 3-damage bolt → Furnace ×2 → Solphim ×2 = 12.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::solphim_mayhem_dominus());
    g.add_card_to_battlefield(0, catalog::furnace_of_rath());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 8, "3 → ×2 (Furnace) → ×2 (Solphim) = 12 damage");

    // Combat: a 5/4 Solphim attack only doubles via Furnace (5 → 10), not Solphim.
    let mut g2 = two_player_game();
    let solphim = g2.add_card_to_battlefield(0, catalog::solphim_mayhem_dominus());
    g2.add_card_to_battlefield(0, catalog::furnace_of_rath());
    g2.clear_sickness(solphim);
    cr_advance_to(&mut g2, TurnStep::DeclareAttackers);
    g2.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: solphim, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g2);
    cr_advance_to(&mut g2, TurnStep::PostCombatMain);
    assert_eq!(g2.players[1].life, 10, "5 combat → ×2 Furnace only = 10");
}

/// CR 122.1 / 702.12 — an indestructible counter (Solphim's activated ability)
/// replaces destruction: the permanent survives a "destroy" effect.
#[test]
fn cr_122_1_indestructible_counter_survives_destroy() {
    let mut g = two_player_game();
    let solphim = g.add_card_to_battlefield(1, catalog::solphim_mayhem_dominus());
    g.battlefield_find_mut(solphim).unwrap()
        .add_counters(CounterType::Indestructible, 1);
    let kill = g.add_card_to_hand(0, catalog::sephiroths_intervention()); // destroy target creature
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: kill, target: Some(Target::Permanent(solphim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast destroy");
    drain_stack(&mut g);
    assert!(g.battlefield_find(solphim).is_some(), "indestructible counter saved it");
}

// ── CR 502.3 — untap-step land lock (Bontu's Last Reckoning) ────────────────
/// CR 502.3 — Bontu's Last Reckoning keeps the caster's lands from untapping
/// for one untap step, then the lock lifts.
#[test]
fn cr_502_3_bontus_lands_skip_one_untap_step() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::bontus_last_reckoning());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bontu's Last Reckoning");
    drain_stack(&mut g);
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(land).unwrap().tapped, "land stays tapped this untap step");
    g.do_untap();
    assert!(!g.battlefield_find(land).unwrap().tapped, "lock is one-shot — untaps next step");
}

// ── CR 611.2 — per-turn spell-cast locks by type ───────────────────────────
/// CR 611.2 — Deafening Silence allows only one noncreature spell per turn but
/// leaves creature spells untouched.
#[test]
fn cr_611_2_deafening_silence_locks_only_noncreature_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::deafening_silence());
    let b1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let b2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: b1, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("first noncreature spell ok");
    drain_stack(&mut g);
    assert!(matches!(
        g.perform_action(GameAction::CastSpell {
            card_id: b2, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }),
        Err(GameError::SpellLimitReached)
    ), "second noncreature spell barred");
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("creature spell unaffected by Deafening Silence");
}

// ── CR 701.16 — targeted sacrifice fires sacrifice + death triggers ─────────
/// CR 701.16 — Effect::SacrificePermanent is a genuine sacrifice: a creature
/// sacrificed this way fires CreatureDied, so a death payoff (Harvester of
/// Souls) sees it. Footsteps of the Goryo sacrifices its reanimated creature
/// at the end step.
#[test]
fn cr_701_16_targeted_sacrifice_fires_death_triggers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::harvester_of_souls());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::footsteps_of_the_goryo());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.add_card_to_library(0, catalog::island());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(dead)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Footsteps of the Goryo");
    drain_stack(&mut g);
    let h = g.players[0].hand.len();
    // Accept Harvester's "may draw" when the reanimated creature is sacrificed.
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    while g.step != crabomination::game::types::TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_none(), "reanimated creature sacrificed at end step");
    assert_eq!(g.players[0].hand.len(), h + 1, "the sacrifice fired Harvester's death draw");
}

// ── CR 105.2c / 111.4 — token color from its color indicator ────────────────
/// A token has no mana cost, so its color comes from the effect that creates
/// it, modeled as a color indicator. Color-filtered effects therefore see a
/// white Soldier token (Raise the Alarm) as white and a colorless Spirit token
/// (Sokenzan) as colorless.
#[test]
fn cr_105_2c_token_color_from_indicator() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let alarm = g.add_card_to_hand(0, catalog::raise_the_alarm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: alarm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Raise the Alarm");
    drain_stack(&mut g);
    let soldier = g.battlefield.iter()
        .find(|c| c.definition.name == "Soldier" && c.controller == 0).expect("soldier token").id;
    assert!(g.evaluate_requirement_static(&R::HasColor(Color::White), &Target::Permanent(soldier), 0, None),
        "white Soldier token is white");
    assert!(!g.evaluate_requirement_static(&R::HasColor(Color::Blue), &Target::Permanent(soldier), 0, None),
        "white Soldier token is not blue");

    let soken = g.add_card_to_hand(0, catalog::sokenzan_crucible_of_defiance());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: soken, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("channel Sokenzan");
    drain_stack(&mut g);
    let spirit = g.battlefield.iter()
        .find(|c| c.definition.name == "Spirit" && c.controller == 0).expect("spirit token").id;
    assert!(!g.evaluate_requirement_static(&R::HasColor(Color::Red), &Target::Permanent(spirit), 0, None),
        "colorless Spirit token has no color");
}

// ── CR 700.5 — devotion counts mana pips, not token color ───────────────────
/// A colored token *is* its color (CR 105.2c) but contributes nothing to
/// devotion, which counts colored mana symbols in mana costs (CR 700.5). The
/// color-indicator change must not leak into the devotion tally.
#[test]
fn cr_700_5_devotion_ignores_colored_tokens() {
    use crabomination::card::SelectionRequirement as R;
    let mut g = two_player_game();
    let alarm = g.add_card_to_hand(0, catalog::raise_the_alarm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: alarm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Raise the Alarm");
    drain_stack(&mut g);
    let soldier = g.battlefield.iter()
        .find(|c| c.definition.name == "Soldier" && c.controller == 0).expect("soldier").id;
    assert!(g.evaluate_requirement_static(&R::HasColor(Color::White), &Target::Permanent(soldier), 0, None),
        "the token is white");
    assert_eq!(g.devotion_to(0, &[Color::White]), 0, "tokens add no devotion (no mana pips)");
}

// ── CR 702.16e — protection prevents a colored token's combat damage ─────────
/// A creature with protection from white takes no combat damage from a white
/// token it blocks (CR 702.16e), so the protection reads the token's color.
#[test]
fn cr_702_16e_protection_prevents_colored_token_damage() {
    let mut g = two_player_game();
    let alarm = g.add_card_to_hand(0, catalog::raise_the_alarm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: alarm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Raise the Alarm");
    drain_stack(&mut g);
    let soldier = g.battlefield.iter()
        .find(|c| c.definition.name == "Soldier" && c.controller == 0).expect("soldier").id;
    g.clear_sickness(soldier);
    // P1's 2/1 has protection from white; it survives blocking the white token.
    let cav = g.add_card_to_battlefield(1, catalog::stillmoon_cavalier());
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: soldier, target: AttackTarget::Player(1),
    }])).expect("attack with the token");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass to blockers");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(cav, soldier)])).expect("block");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(cav).is_some(), "pro-white blocker took no damage from the white token");
}

// ── CR 701.54 — The Ring tempts you / Ring-bearer ───────────────────────────

/// CR 701.54a — temptation designates a creature the player controls as their
/// Ring-bearer (the engine auto-picks the strongest).
#[test]
fn cr_701_54a_ring_tempts_designates_a_bearer() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ev = vec![];
    g.ring_tempts(0, &mut ev);
    assert_eq!(g.players[0].ring_temptations, 1);
    assert_eq!(g.effective_ring_bearer(0), Some(bear));
}

/// CR 701.54c — at one+ temptation the Ring-bearer can't be blocked by a
/// creature with greater power; at four+ it drains each opponent 3 on combat
/// damage to a player.
#[test]
fn cr_701_54c_ring_bearer_evasion_and_drain() {
    let mut g = two_player_game();
    let bearer = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(bearer);
    let big = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    // Cover the level-2 attack loot so the bearer doesn't deck out.
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    g.add_card_to_hand(0, catalog::forest());
    let mut ev = vec![];
    for _ in 0..4 { g.ring_tempts(0, &mut ev); } // level 4
    assert!(!g.blocker_can_block_attacker(big, bearer), "greater-power blocker barred");
    let before = g.players[1].life;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bearer, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[1].life, before - 2 - 3, "2 combat + 3 Ring drain");
}

// ── CR 701.47 — Amass ───────────────────────────────────────────────────────

/// CR 701.47a — amass with no Army first mints a 0/0 Army token, then loads it
/// with N +1/+1 counters (Easterling Vanguard's death trigger amasses Orcs 1).
#[test]
fn cr_701_47a_amass_mints_and_grows_an_army() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::easterling_vanguard());
    let ev = g.remove_to_graveyard_with_triggers(v);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let army = g.battlefield.iter().find(|c| c.controller == 0
        && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Army))
        .expect("Army token minted");
    let cp = g.compute_battlefield();
    let acp = cp.iter().find(|c| c.id == army.id).unwrap();
    assert_eq!((acp.power, acp.toughness), (1, 1), "0/0 Army + one +1/+1 counter");
}

// ── CR 701.54c — the Ring's level-1 emblem makes the Ring-bearer legendary ────

/// CR 701.54c — at level 1+, your Ring-bearer is legendary. Modeled by a
/// synthetic `Modification::AddSupertype(Legendary)` layer-4 effect (CR 205.4b).
#[test]
fn cr_701_54c_ring_bearer_is_legendary() {
    use crabomination::card::Supertype;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // nonlegendary
    g.ring_tempts(0, &mut vec![]);
    assert_eq!(g.effective_ring_bearer(0), Some(bear));
    let cp = g.compute_battlefield();
    assert!(cp.iter().find(|c| c.id == bear).unwrap().supertypes.contains(&Supertype::Legendary),
        "Ring-bearer gains the Legendary supertype");
}

// ── CR 509.1c — "can't block unless you control N+ [filter]" (block-only) ──────

/// CR 509.1c — a block restriction that gates blocking only (Olog-hai Crusher:
/// "can't block unless you control a Goblin or Orc"), leaving attacking free.
#[test]
fn cr_509_1c_block_only_restriction() {
    let mut g = two_player_game();
    let olog = g.add_card_to_battlefield(1, catalog::olog_hai_crusher());
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.blocker_can_block_attacker(olog, atk), "no Goblin/Orc → can't block");
    g.add_card_to_battlefield(1, catalog::goblin_guide());
    assert!(g.blocker_can_block_attacker(olog, atk), "with a Goblin → can block");
}

// ── CR 701.12-style one-sided power damage ────────────────────────────────────

/// One-sided "deals damage equal to its power" (Stew the Coneys). The source
/// takes no back-swing; deathtouch/lifelink ride the source via the funnel.
#[test]
fn one_sided_power_damage_no_backswing() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::stew_the_coneys());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(foe)], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == foe), "enemy took 3 and died");
    assert_eq!(g.battlefield_find(mine).unwrap().damage, 0, "our creature takes no back-swing");
}

// ── CR 120.10 — excess damage ─────────────────────────────────────────────────

/// Excess damage (CR 120.10) is the amount beyond lethal; it accumulates per
/// resolution and resets between resolutions, gating Orbital Plunge's Lander.
#[test]
fn cr_120_10_excess_damage_tracked_per_resolution() {
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::hill_giant());
    // 6 to a 2/2 → 4 excess.
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext::for_ability(src, 0, Some(Target::Permanent(small)));
    g.resolve_effect(&Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(6) }, &ctx).unwrap();
    assert_eq!(g.excess_damage_this_resolution, 4, "6 to a 2/2 → 4 excess");
    // 6 to a 9/9 → no excess, and the counter reset between resolutions.
    let big = g.add_card_to_battlefield(1, catalog::bygone_colossus());
    let ctx = EffectContext::for_ability(src, 0, Some(Target::Permanent(big)));
    g.resolve_effect(&Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(6) }, &ctx).unwrap();
    assert_eq!(g.excess_damage_this_resolution, 0, "6 to a 9/9 → no excess");
}

/// CR 120.10 — `Value::ExcessDamageDealtThisResolution` reads the running
/// excess so "gain life equal to the excess damage dealt this way" works
/// within one resolution (Razor Rings).
#[test]
fn cr_120_10_excess_damage_value_reads_running_total() {
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::hill_giant());
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let life = g.players[0].life;
    let ctx = EffectContext::for_ability(src, 0, Some(Target::Permanent(small)));
    g.resolve_effect(&Effect::Seq(vec![
        Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(5) }, // 3 excess
        Effect::GainLife { who: Selector::You, amount: Value::ExcessDamageDealtThisResolution },
    ]), &ctx).unwrap();
    assert_eq!(g.players[0].life, life + 3, "gained life equal to the 3 excess damage");
}

// ── CR 207.2c / 700.11 — Descend (LCI ability word) ──────────────────────────

/// "Descend 8" reads "eight or more permanent cards in your graveyard"
/// (CR 207.2c ability word). Crafting Waterlogged Hulk into Watertight Gondola
/// grants Unblockable only once the controller's graveyard holds 8 permanent
/// cards; instant/sorcery cards there don't count.
#[test]
fn descend_8_grants_unblockable_only_at_eight_permanent_cards() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let hulk = g.add_card_to_battlefield(0, catalog::waterlogged_hulk());
    g.add_card_to_battlefield(0, catalog::island()); // Island to exile as the craft cost
    // Seven permanent cards + one instant in the graveyard → descend 7.
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // not a permanent card
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hulk, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("craft into gondola");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(hulk).unwrap().definition.name, "Watertight Gondola");
    assert!(
        !g.computed_permanent(hulk).unwrap().keywords.contains(&Keyword::Unblockable),
        "descend 7 (instant doesn't count) → still blockable",
    );
    // Add an eighth permanent card → descend 8 flips the static on.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    assert!(
        g.computed_permanent(hulk).unwrap().keywords.contains(&Keyword::Unblockable),
        "descend 8 → unblockable",
    );
}

// ── CR 702.169 — Craft ────────────────────────────────────────────────────────

/// Craft (CR 702.169) is a sorcery-speed activated ability that exiles the
/// source and other objects, returning the source transformed.
#[test]
fn cr_702_169_craft_exiles_and_returns_transformed() {
    let mut g = two_player_game();
    let blade = g.add_card_to_battlefield(0, catalog::tithing_blade());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: blade, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("craft");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == fodder), "creature exiled as cost");
    assert_eq!(g.battlefield_find(blade).unwrap().definition.name, "Consuming Sepulcher");
}

// ── CR 104.3c / 704.5a — losing on an empty-library draw ─────────────────────

/// A player who has attempted to draw from an empty library since the last
/// SBA check loses the game (CR 104.3c via 704.5c).
#[test]
fn cr_704_5c_drawing_from_empty_library_loses() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    let mut events = vec![];
    assert!(!g.draw_one(0, &mut events), "draw from empty library fails");
    g.lose_to_empty_draw(0);
    g.check_state_based_actions();
    assert!(g.players[0].eliminated, "empty-library draw eliminates the player");
    assert!(g.is_game_over(), "game ends — the other player wins");
}

// ── CR 603.7 — reflexive "when you do" triggered abilities ───────────────────

/// CR 603.7 — a reflexive ability set up by "you may pay {2}. When you do, …"
/// chooses its targets when it triggers (after the cost), not when the ETB
/// trigger is put on the stack. `Effect::Reflexive` is opaque to the cast/
/// trigger-time target walk; Itzquinth's bite resolves only after the {2}.
#[test]
fn cr_603_7_reflexive_payoff_targets_after_cost() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let itz = g.add_card_to_battlefield(0, catalog::itzquinth_firstborn_of_gishath());
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.fire_self_etb_triggers(itz, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(prey).is_none(), "reflexive bite resolved after the {{2}} was paid");
}

// ── CR 701.57 — Discover ─────────────────────────────────────────────────────

/// CR 701.57 — discover N exiles from the top until a nonland card with mana
/// value ≤ N, then casts it for free (or puts it into hand). Trumpeting
/// Carnosaur discovers 5 on ETB.
#[test]
fn cr_701_57_discover_digs_and_casts() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 ≤ 5 → discovered
    let carno = g.add_card_to_battlefield(0, catalog::trumpeting_carnosaur());
    let lib_before = g.players[0].library.len();
    g.fire_self_etb_triggers(carno, 0);
    drain_stack(&mut g);
    assert!(g.players[0].library.len() < lib_before, "discover dug into the library");
    // The MV-2 nonland card was discovered — cast for free or put into hand.
    let found = g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Grizzly Bears")
        || g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears");
    assert!(found, "discovered card moved out of the library (cast or to hand)");
}

/// CR 702.56 — Forecast: a hand-activated ability usable only during the
/// owner's upkeep, only once each turn. Pride of the Clouds mints a Bird.
#[test]
fn cr_702_56_forecast_once_per_turn_in_upkeep() {
    let mut g = two_player_game();
    let pride = g.add_card_to_hand(0, catalog::pride_of_the_clouds());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Outside the upkeep the forecast condition fails.
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: pride, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).is_err(), "forecast can't fire outside upkeep");
    // In your upkeep it resolves; the card stays in hand.
    g.step = TurnStep::Upkeep;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pride, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("forecast in upkeep");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == pride), "forecast card stays in hand");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Bird").count(), 1, "made a Bird");
    // Second activation the same turn is rejected (once each turn).
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: pride, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).is_err(), "once each turn");
}

/// CR 603.4 — a turn-scoped "whenever [watched creature] deals damage this
/// turn" delayed trigger fires on combat damage too (not just noncombat) and
/// expires at cleanup. Paladin of Prahv's Forecast watches a creature.
#[test]
fn cr_603_4_watched_creature_damage_fires_on_combat_and_expires() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(bear);
    let paladin = g.add_card_to_hand(0, catalog::paladin_of_prahv());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: paladin, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None, mode: None,
    }).expect("forecast");
    drain_stack(&mut g);
    // Combat damage from the watched creature gains us that much life.
    let life0 = g.players[0].life;
    g.fire_combat_damage_to_player_triggers(bear, 1, 2);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 2, "combat damage from the watched creature gained life");
    // After cleanup the watcher is gone.
    g.do_cleanup(&mut Vec::new());
    let life1 = g.players[0].life;
    let mut ev = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(1), 2, Some(bear), &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life1, "watcher expired at cleanup — no more life gain");
}

/// CR 615.7 — "Prevent all damage a source of your choice would deal this turn"
/// (Prahv, Spires of Order). The chosen source deals no damage afterward.
#[test]
fn cr_615_7_prevent_all_damage_from_chosen_source() {
    let mut g = two_player_game();
    let prahv = g.add_card_to_battlefield(0, catalog::prahv_spires_of_order());
    g.clear_sickness(prahv);
    // A creature whose damage we'll prevent; script the source choice to it.
    let ogre = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Cards(vec![ogre]),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: prahv, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate Prahv's prevention");
    drain_stack(&mut g);
    let life1 = g.players[0].life;
    let mut ev = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0), 2, Some(ogre), &mut ev);
    assert_eq!(g.players[0].life, life1, "the chosen source's damage was prevented");
}

/// CR 701.40 — Explore via a Map token: sacrifice the Map to explore a
/// creature; a land reveal goes to hand.
#[test]
fn cr_701_40_map_token_explore() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // A Map maker: cast Spyglass Siren (ETB makes a Map).
    let siren = g.add_card_to_hand(0, catalog::spyglass_siren());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: siren, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast siren");
    drain_stack(&mut g);
    let map = g.battlefield.iter().find(|c| c.definition.name == "Map").map(|c| c.id).expect("Map token");
    g.add_card_to_library(0, catalog::forest()); // land on top → explore to hand
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: map, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None, mode: None,
    }).expect("sac Map to explore");
    drain_stack(&mut g);
    assert!(g.battlefield_find(map).is_none(), "Map sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "explored land to hand");
}

/// CR 510.1c / 702.2e — a trample + deathtouch attacker assigns only 1 (lethal
/// under deathtouch) to the blocker and tramples the rest to the player.
#[test]
fn cr_510_1c_deathtouch_trample_assigns_one() {
    use crabomination::game::types::Attack;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::pest_reaper_b120()); // 4/4 trample+deathtouch
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(1) }];
    g.set_block_map([(blocker, attacker)]);
    g.step = TurnStep::CombatDamage;
    g.active_player_idx = 0;
    let life_before = g.players[1].life;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(blocker).is_none(), "blocker dies to deathtouch");
    assert_eq!(g.players[1].life, life_before - 3, "1 lethal to blocker, 3 trample over");
}

// ── CR 702.122 — Crew ───────────────────────────────────────────────────────

/// CR 702.122 — tapping creatures with total power ≥ the crew cost turns a
/// Vehicle into an artifact creature until end of turn.
#[test]
fn cr_702_122_crew_animates_vehicle() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(0, catalog::broadcast_rambler()); // Crew 1
    let crewer = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(crewer);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(!g.computed_permanent(veh).unwrap().card_types.contains(&CardType::Creature),
        "uncrewed Vehicle is not a creature");
    g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![crewer] })
        .expect("crew");
    assert!(g.computed_permanent(veh).unwrap().card_types.contains(&CardType::Creature),
        "crewed Vehicle becomes an artifact creature");
}

// ── CR 702.171 — Saddle ─────────────────────────────────────────────────────

/// CR 702.171 — tapping creatures with total power ≥ the saddle cost saddles a
/// Mount, enabling its "attacks while saddled" trigger.
#[test]
fn cr_702_171_saddle_enables_attack_trigger() {
    let mut g = two_player_game();
    let mount = g.add_card_to_battlefield(0, catalog::alacrian_jaguar()); // Saddle 1, 4/4
    let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(mount);
    g.clear_sickness(helper);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Saddle { mount, creatures: vec![helper] }).expect("saddle");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: mount, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(mount).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "attacks-while-saddled gave +2/+2");
}

/// CR 702.171 — a Saddle activation records the riders on the Mount
/// (`saddled_by`) for "a creature that saddled it this turn" payoffs, and CR
/// 400.7 clears that record when the Mount leaves the battlefield.
#[test]
fn cr_702_171_saddle_records_riders_reset_on_leave() {
    let mut g = two_player_game();
    let mount = g.add_card_to_battlefield(0, catalog::alacrian_jaguar());
    let rider = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(mount);
    g.clear_sickness(rider);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Saddle { mount, creatures: vec![rider] }).expect("saddle");
    assert_eq!(g.battlefield_find(mount).unwrap().saddled_by, vec![rider]);
    // Bounce and replay the Mount → CR 400.7 fresh object, no riders remembered.
    g.move_card_to_battlefield_for_test(0, catalog::alacrian_jaguar());
    let fresh = g.battlefield.iter().rev().find(|c| c.definition.name == "Alacrian Jaguar").unwrap();
    assert!(fresh.saddled_by.is_empty(), "a fresh Mount object remembers no riders");
}

/// CR 702.9 — a Crew activation records the crewers on the Vehicle
/// (`crewed_by`) so "for each creature that crewed it this turn" payoffs
/// (Luxurious Locomotive) can count them.
#[test]
fn cr_702_9_crew_records_crewers() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(0, catalog::broadcast_rambler()); // Crew 1
    let crewer = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(crewer);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![crewer] }).expect("crew");
    assert_eq!(g.battlefield_find(veh).unwrap().crewed_by, vec![crewer]);
}

/// CR 702.108 — Inspired: "whenever this becomes untapped" fires off a
/// `PermanentUntapped` event (the untap-step turn-based action or any untap
/// effect), the untap sibling of Wylie Duke's tap trigger.
#[test]
fn cr_702_108_inspired_fires_on_untap() {
    let mut g = two_player_game();
    let tromper = g.add_card_to_battlefield(0, catalog::pheres_band_tromper());
    g.battlefield_find_mut(tromper).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentUntapped { card_id: tromper }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(tromper).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Inspired put a +1/+1 counter on untap",
    );
}

// ── CR 702.179 — Start your engines! ─────────────────────────────────────────

/// CR 702.179a — a "Start your engines!" permanent entering sets its
/// controller's speed to 1 (the speed starts at 0).
#[test]
fn cr_702_179_start_your_engines_sets_speed_to_one() {
    let mut g = two_player_game();
    assert_eq!(g.players[0].speed, 0, "no speed before any engine");
    g.move_card_to_battlefield_for_test(0, catalog::glitch_ghost_surveyor());
    drain_stack(&mut g);
    assert_eq!(g.players[0].speed, 1, "Start your engines! set speed to 1");
}

/// CR 702.179c/d — once the engine is running, the active player's speed rises
/// by 1 the first time an opponent loses life on their turn (at most once per
/// turn), and never past the maximum of 4.
#[test]
fn cr_702_179_speed_increments_once_per_turn_capped_at_four() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.players[0].speed = 1; // engine already running
    // First opponent life loss this turn → 1 → 2.
    g.adjust_life(1, -3);
    assert_eq!(g.players[0].speed, 2, "first opp life-loss bumps speed");
    // A second loss the same turn does nothing (once per turn).
    g.adjust_life(1, -3);
    assert_eq!(g.players[0].speed, 2, "only once per turn");
    // From speed 4 it never rises further, even on a fresh turn.
    g.players[0].speed = 4;
    g.players[0].speed_increased_this_turn = false;
    g.adjust_life(1, -3);
    assert_eq!(g.players[0].speed, 4, "capped at maximum speed 4");
    // The active player's *own* life loss never bumps their speed.
    g.players[0].speed = 1;
    g.players[0].speed_increased_this_turn = false;
    g.adjust_life(0, -3);
    assert_eq!(g.players[0].speed, 1, "your own life loss doesn't advance your engine");
}

/// CR 702.177b — an exhaust ability can be activated only once per game. A
/// second activation of Boommobile's exhaust ability is rejected.
#[test]
fn cr_702_177_exhaust_ability_only_once_per_game() {
    let mut g = two_player_game();
    let boom = g.add_card_to_battlefield(0, catalog::boommobile());
    g.priority.player_with_priority = 0;
    let pay = |g: &mut GameState| {
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3); // {X=1}{2}{R}
    };
    pay(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: boom, ability_index: 0, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: Vec::new(), x_value: Some(1), mode: None,
    })
    .expect("first exhaust activation succeeds");
    drain_stack(&mut g);
    pay(&mut g);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: boom, ability_index: 0, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: Vec::new(), x_value: Some(1), mode: None,
        }).is_err(),
        "second exhaust activation is rejected (once per game)"
    );
}

// ── CR 702.2e / 510.1c — trample + deathtouch over multiple blockers ──────────

/// A trample + deathtouch attacker assigns only 1 (the deathtouch lethal) to
/// each blocker, so the rest tramples through (CR 510.1c lethal accounting +
/// CR 702.2e "any nonzero amount is lethal under deathtouch").
fn trample_deathtoucher(power: i32) -> crabomination::card::CardDefinition {
    use crabomination::card::{CardDefinition, CardType, Keyword};
    CardDefinition {
        name: "Test Trample Deathtoucher",
        cost: crabomination::mana::cost(&[crabomination::mana::generic(3), crabomination::mana::g()]),
        card_types: vec![CardType::Creature],
        power,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Deathtouch],
        ..Default::default()
    }
}

#[test]
fn cr_702_2e_trample_deathtouch_assigns_one_per_blocker_then_tramples() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, trample_deathtoucher(4)); // 4/4
    g.clear_sickness(attacker);
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(b1);
    g.clear_sickness(b2);
    let life_before = g.players[1].life;

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, attacker), (b2, attacker)])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();

    // 1 lethal (deathtouch) to each blocker → both die; 4 - 1 - 1 = 2 tramples over.
    assert!(g.battlefield_find(b1).is_none() && g.battlefield_find(b2).is_none(),
        "deathtouch kills both blockers with 1 damage each");
    assert_eq!(g.players[1].life, life_before - 2, "the remaining 2 power tramples through");
}

// ── CR 305.7 / 613 — a type-set effect swaps which tribal lord applies ─────────

/// Turn to Frog's `BecomeCreatureType` replaces the creature's types (layer 4),
/// so a non-Frog joins a Frog lord's umbrella — checked through the layer
/// pipeline (CR 613 ordering: type set at L4 feeds the L7c anthem).
#[test]
fn cr_305_7_type_change_swaps_lord_applicability() {
    use crabomination::card::{CardType, CreatureType, StaticAbility, StaticEffect, SelectionRequirement, Selector};
    let mut g = two_player_game();
    // A Frog lord: "Other Frogs you control get +1/+1."
    let lord = crabomination::card::CardDefinition {
        name: "Test Frog Lord",
        cost: crabomination::mana::cost(&[crabomination::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: crabomination::card::Subtypes {
            creature_types: vec![CreatureType::Frog], ..Default::default()
        },
        power: 2, toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Other Frogs you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Frog)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: 1, toughness: 1,
            },
        }],
        ..Default::default()
    };
    g.add_card_to_battlefield(0, lord);
    // A vanilla 2/2 that is NOT a Frog — unaffected by the lord at first.
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bears).unwrap().power, 2, "non-Frog unbuffed");

    // Turn the bears into a 1/1 Frog — sets its type at layer 4.
    let spell = g.add_card_to_hand(0, catalog::turn_to_frog());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Turn to Frog castable");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bears).unwrap();
    assert!(cp.subtypes.creature_types == vec![CreatureType::Frog], "now a Frog");
    // Base 1/1 (Turn to Frog) + the Frog lord's +1/+1 → 2/2.
    assert_eq!((cp.power, cp.toughness), (2, 2), "the Frog lord now buffs the new Frog");
}

// ── CR 604.3 — characteristic-defining P/T recomputes live ─────────────────────

#[test]
fn cr_604_3_lhurgoyf_pt_tracks_graveyard_creatures() {
    // A CDA P/T (Lhurgoyf = creature cards in all graveyards / +1 toughness) is
    // recomputed continuously, so adding a creature card to a graveyard grows it.
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lhurgoyf());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 1), "empty graveyards → 0/1");
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3), "two creature cards across graveyards → 2/3");
}

// ── CR 108.3 / 800.4a — ownership independent of control ───────────────────────

#[test]
fn cr_108_3_gain_control_of_permanents_you_own() {
    // Gruul Charm mode 1: "Gain control of all permanents you own." A creature
    // you OWN but an opponent CONTROLS returns to you (SelectionRequirement::
    // OwnedByYou keys off owner, not controller — CR 108.3).
    use crabomination::card::Effect;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Hand it to the opponent's control while P0 keeps ownership.
    g.battlefield_find_mut(bear).unwrap().controller = 1;
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 1);
    let Effect::ChooseMode(m) = catalog::gruul_charm().effect else { panic!() };
    g.resolve_effect(&m[1], &crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None)).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0,
        "you regain control of a permanent you own");
}

// ── CR 702.85 — Cascade granted to other spells (The First Sliver) ─────────────

#[test]
fn cr_702_85_first_sliver_grants_sliver_spells_cascade() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Library top: a nonland the cascade can hit (MV 2 < Megantic Sliver's MV 6).
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    // The First Sliver on the battlefield grants your Sliver spells cascade.
    g.add_card_to_battlefield(0, catalog::the_first_sliver());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let meg = g.add_card_to_hand(0, catalog::megantic_sliver());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: meg, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Megantic Sliver castable for {5}{G}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bears),
        "the granted cascade casts the lower-MV card onto the battlefield");
}

// ── CR 506 — skip the active player's combat phase (Stonehorn Dignitary) ───────

#[test]
fn cr_506_active_player_skips_their_combat_phase() {
    // A banked skip charge sends Begin Combat straight to the postcombat main:
    // no declare/damage steps occur.
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.players[0].skip_next_combat = 1;
    g.step = TurnStep::PreCombatMain;
    g.advance_step(Vec::new()).expect("advance from precombat main");
    assert_eq!(g.step, TurnStep::PostCombatMain, "combat phase skipped entirely");
    assert_eq!(g.players[0].skip_next_combat, 0, "charge consumed");
}

#[test]
fn cr_506_skip_only_eats_one_combat() {
    // A second turn's combat is unaffected once the single charge is spent.
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.players[0].skip_next_combat = 1;
    g.step = TurnStep::PreCombatMain;
    g.advance_step(Vec::new()).expect("skip the first combat");
    assert_eq!(g.step, TurnStep::PostCombatMain);
    // Next time we reach Begin Combat there's no charge → normal combat.
    g.step = TurnStep::PreCombatMain;
    g.advance_step(Vec::new()).expect("advance again");
    assert_eq!(g.step, TurnStep::BeginCombat, "no charge left, combat proceeds");
}

// ── CR 602.5b — remove-a-counter activation cost (quest cycle) ─────────────────

#[test]
fn cr_602_5b_quest_activation_needs_its_counters() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::quest_for_the_gravelord());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // Only two counters — short of the three the cost demands.
    g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 2);
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: quest, ability_index: 0, target: None, additional_targets: Vec::new(),
        x_value: None, mode: None,
    });
    assert!(res.is_err(), "can't pay the remove-three-counters cost");
    // Top up and it activates, consuming the counters via sacrifice.
    g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: quest, ability_index: 0, target: None, additional_targets: Vec::new(),
        x_value: None, mode: None,
    }).expect("now payable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Zombie Giant"));
}

// ── CR 605.1c — "Spend this mana only to activate abilities" ────────────────

/// Omen Hawker's mana funds ability activations but is rejected for spell
/// casts (the new `SpendRestriction::AbilitiesOnly`, CR 605.1c).
#[test]
fn cr_605_1c_abilities_only_mana_funds_abilities_not_spells() {
    use crabomination::mana::{ManaCost, SpellKind};
    let mut g = two_player_game();
    let hawker = g.add_card_to_battlefield(0, catalog::omen_hawker());
    g.clear_sickness(hawker);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hawker, ability_index: 0, target: None, additional_targets: Vec::new(),
        x_value: None, mode: None,
    }).expect("tap for {C}{U}");
    let pool = &mut g.players[0].mana_pool;
    assert_eq!(pool.restricted_total(), 2, "two restricted mana (C and U)");
    let one = ManaCost::new(vec![crabomination::mana::generic(1)]);
    assert!(pool.pay_for_spell(&one, &SpellKind::default()).is_err(), "can't fund a spell");
    let ability = SpellKind { activating_ability: true, ..Default::default() };
    assert!(pool.pay_for_spell(&one, &ability).is_ok(), "funds an ability activation");
}

// ── CR 604.3 — characteristic-defining P/T from the controller's graveyard ──

/// Nethergoyf's `*/1+*` reads card types in *its controller's own* graveyard
/// (CR 604.3 — `DynamicPt::CardTypesInControllerGraveyard`), and an opponent's
/// graveyard doesn't feed it.
#[test]
fn cr_604_3_nethergoyf_counts_only_your_graveyard_types() {
    let mut g = two_player_game();
    let goyf = g.add_card_to_battlefield(0, catalog::nethergoyf());
    // Opponent's graveyard types must NOT count.
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let v = g.compute_battlefield();
    let c = v.iter().find(|c| c.id == goyf).unwrap();
    assert_eq!((c.power, c.toughness), (0, 1), "opponent graveyard ignored");
    // Your own graveyard with three card types → 3/4.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());   // Creature
    g.add_card_to_graveyard(0, catalog::lightning_bolt());  // Instant
    g.add_card_to_graveyard(0, catalog::forest());          // Land
    let v = g.compute_battlefield();
    let c = v.iter().find(|c| c.id == goyf).unwrap();
    assert_eq!((c.power, c.toughness), (3, 4), "three of your card types → 3/4");
}

// ── CR 509.1b — a creature that "can't block" is an illegal blocker ─────────

/// Hazardous Blast grants every opposing creature "can't block this turn"; the
/// engine then refuses to declare such a creature as a blocker (CR 509.1b).
#[test]
fn cr_509_1b_cant_block_grant_rejects_the_blocker() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 survives the ping
    let blast = g.add_card_to_hand(0, catalog::hazardous_blast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: blast, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hazardous Blast");
    drain_stack(&mut g);
    assert!(g.computed_permanent(wall).unwrap().keywords.contains(&Keyword::CantBlock));
    // Attack with a creature; the CantBlock wall can't be declared as a blocker.
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareBlockers;
    g.block_map.clear();
    let err = g.perform_action(GameAction::DeclareBlockers(vec![(wall, attacker)]));
    assert!(err.is_err(), "a can't-block creature is an illegal blocker");
}

/// CR 701.67 — a noncreature artifact you control is a legal waterbend helper
/// (Convoke only takes creatures; waterbend takes artifacts too).
#[test]
fn cr_701_67_waterbend_accepts_an_artifact_helper() {
    let mut g = two_player_game();
    // Four creature helpers + one artifact helper cover the {5}.
    let mut helpers = Vec::new();
    for _ in 0..4 {
        let h = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(h);
        helpers.push(h);
    }
    let lyre = g.add_card_to_battlefield(0, catalog::entrancing_lyre());
    g.clear_sickness(lyre);
    helpers.push(lyre);
    let spirit = g.add_card_to_hand(0, catalog::benevolent_river_spirit());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpellWaterbend {
        card_id: spirit, target: None, additional_targets: vec![], mode: None, x_value: None,
        helpers,
    }).expect("artifact + creatures pay the waterbend {5}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == spirit));
    assert!(g.battlefield_find(lyre).unwrap().tapped, "the artifact helper tapped");
}

/// CR 701.67 — paying an optional "you may waterbend {N}" cost sets the
/// provenance flag that `Predicate::SpellWasWaterbend` reads.
#[test]
fn cr_701_67_optional_waterbend_records_provenance() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let h1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let h2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(h1); g.clear_sickness(h2);
    let lesson = g.add_card_to_hand(0, catalog::waterbending_lesson()); // discard unless waterbent
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellWaterbend {
        card_id: lesson, target: None, additional_targets: vec![], mode: None, x_value: None,
        helpers: vec![h1, h2],
    }).expect("pay the optional waterbend");
    drain_stack(&mut g);
    // Provenance honored: drew 3, no discard (net +3 minus the lesson leaving hand).
    assert_eq!(g.players[0].hand.len(), before - 1 + 3, "SpellWasWaterbend → discard skipped");
}

// ── CR 115.1a / 601.2c — a spell may target objects of different kinds in
// distinct slots (a permanent in one slot, a player in another). The
// `ControlledBy { who: Target(n) }` selector declares slot n as a player
// target (How to Start a Riot — "creatures target player controls get +2/+0").
#[test]
fn cr_115_spell_targets_a_creature_and_a_player() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let riot = g.add_card_to_hand(0, catalog::how_to_start_a_riot());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: riot,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    }).expect("two-kind targeting accepted");
    drain_stack(&mut g);
    assert!(g.computed_permanent(mine).unwrap().keywords.contains(&crabomination::card::Keyword::Menace));
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 4, "the player slot resolved");
}

// ── CR 506.5 — a creature "attacks alone" only when it's the sole attacker.
// Team Avatar's lone-attacker pump must NOT fire when two creatures attack.
#[test]
fn cr_506_5_attacks_alone_requires_a_sole_attacker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::team_avatar());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ])).expect("two attackers");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(a).unwrap().power, 2, "no lone-attacker pump with two attackers");
}

// ── CR 601.2c / 115.1 — a multi-target spell binds each slot to its own legal
// object kind: Sokka's Haiku counters a *spell* (slot 0) and untaps a *land*
// (slot 1).
#[test]
fn cr_601_2c_multitarget_spell_and_land_slots() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts");
    let land = g.add_card_to_battlefield(0, catalog::island());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    let haiku = g.add_card_to_hand(0, catalog::sokkas_haiku());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: haiku,
        target: Some(Target::Permanent(spell)),
        additional_targets: vec![Target::Permanent(land)],
        mode: None,
        x_value: None,
    }).expect("counter-spell + untap-land slots accepted");
    drain_stack(&mut g);
    assert!(g.battlefield_find(spell).is_none(), "spell slot countered");
    assert!(!g.battlefield_find(land).unwrap().tapped, "land slot untapped");
}


// ── CR 701.66 — Earthbend ──────────────────────────────────────────────────

/// CR 701.66a — earthbending a land you control turns it into a 0/0 land
/// creature with haste, puts N +1/+1 counters on it, and (CR 701.66a) returns
/// it tapped when it dies. Exercised through Earth Kingdom General (earthbend
/// 2 on ETB).
#[test]
fn cr_701_66a_earthbend_lifecycle_and_return_on_death() {
    use crabomination::card::{CardType, Keyword};
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let ekg = g.add_card_to_battlefield(0, catalog::earth_kingdom_general());
    g.fire_self_etb_triggers(ekg, 0);
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature), "land became a creature");
    assert!(cp.card_types.contains(&CardType::Land), "stays a land");
    assert!(cp.keywords.contains(&Keyword::Haste), "gains haste");
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "earthbend 2 counters");
    // Kill it → returns to the battlefield tapped.
    g.battlefield_find_mut(land).unwrap().damage = 2; // 0/0 + 2 counters = 2/2
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let back = g.battlefield_find(land).expect("returned to battlefield");
    assert!(back.tapped, "returns tapped (CR 701.66a)");
}

/// CR 704.5f — an animated land reduced to 0 toughness (its earthbend counters
/// removed) is put into its owner's graveyard as a state-based action, even
/// though its printed card is a land.
#[test]
fn cr_704_5f_zero_toughness_animated_land_dies() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let ekg = g.add_card_to_battlefield(0, catalog::earth_kingdom_general());
    g.fire_self_etb_triggers(ekg, 0); // earthbend 2 → 2/2 land creature
    drain_stack(&mut g);
    // Strip the +1/+1 counters → 0/0 computed creature.
    g.battlefield_find_mut(land).unwrap().remove_counters(CounterType::PlusOnePlusOne, 2);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    // The earthbend return brings it back tapped (it "died"), so it's a land
    // again — the key assertion is that the 0/0 creature didn't linger.
    assert!(!g.computed_permanent(land)
        .map(|c| c.card_types.contains(&crabomination::card::CardType::Creature) && c.toughness == 0)
        .unwrap_or(false), "no 0/0 creature lingers on the battlefield");
}

/// Guard: the computed-type SBA death check doesn't kill a plain (non-animated)
/// land, whose computed toughness is 0 but which isn't a creature.
#[test]
fn cr_704_5f_plain_land_is_not_a_creature_and_survives() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some(), "a plain land never dies to SBA");
}

/// War Balloon with 3 fire counters becomes a creature (layer 4) and a +1/+1
/// counter still raises its printed 4/3 to 5/4 (layer 7).
#[test]
fn cr_613_counter_gated_creature_type_composes_with_pt() {
    use crabomination::card::{CardType, CounterType};
    let mut g = two_player_game();
    let wb = g.add_card_to_battlefield(0, catalog::war_balloon());
    let c = g.battlefield_find_mut(wb).unwrap();
    c.add_counters(CounterType::Fire, 3);
    c.add_counters(CounterType::PlusOnePlusOne, 1);
    let cp = g.computed_permanent(wb).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature), "layer-4 AddCardType from 3 fire counters");
    assert_eq!((cp.power, cp.toughness), (5, 4), "printed 4/3 + a +1/+1 counter");
}

/// CR 103.4 — "N more than your starting life total" reads the *actual*
/// starting life, not a hardcoded 20+N. At a 40-life start Speaker of the
/// Heavens needs 47, not 27.
#[test]
fn cr_103_4_above_starting_reads_actual_starting_life() {
    let mut g = two_player_game();
    let speaker = g.add_card_to_battlefield(0, catalog::speaker_of_the_heavens());
    g.clear_sickness(speaker);
    g.players[0].starting_life = 40;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // 46 = start+6 — still below the +7 gate even though it's well over 27.
    g.players[0].life = 46;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: speaker, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).is_err(), "blocked at only +6 above a 40 start");
    g.battlefield_find_mut(speaker).unwrap().tapped = false;
    g.players[0].life = 47; // start+7
    g.perform_action(GameAction::ActivateAbility {
        card_id: speaker, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("fires at +7 above a 40 start");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Angel"), "Angel made at +7");
}

/// CR 614 / 119.10 — a life-gain multiplier applies before an additive bonus,
/// and neither applies to a gain of 0 (no life-gain event occurs).
#[test]
fn cr_614_life_gain_multiplier_precedes_bonus() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rhox_faithmender()); // x2
    g.add_card_to_battlefield(0, catalog::angel_of_vitality()); // +1
    let life = g.players[0].life;
    // 3 → ×2 = 6 → +1 = 7.
    g.adjust_life(0, 3);
    assert_eq!(g.players[0].life, life + 7, "multiplier then bonus");
    // A gain of 0 is not a gain — the replacements leave it at 0 (CR 119.10).
    let now = g.players[0].life;
    g.adjust_life(0, 0);
    assert_eq!(g.players[0].life, now, "0-gain untouched by ×2/+1");
}

/// CR 121.2b — "can't draw more than one card each turn" truncates a multi-draw
/// to the remaining allowance (Spirit of the Labyrinth).
#[test]
fn cr_121_2b_draw_cap_truncates_multidraw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spirit_of_the_labyrinth());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let div = g.add_card_to_hand(0, catalog::divination()); // draw 2
    g.players[0].cards_drawn_this_turn = 0;
    let hand = g.players[0].hand.len(); // includes Divination
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: div, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Divination");
    drain_stack(&mut g);
    // Divination left hand (-1), drew only 1 of its 2 (cap) → net hand unchanged.
    assert_eq!(g.players[0].hand.len(), hand, "draw-2 capped to 1 under the Spirit");
    assert_eq!(g.players[0].cards_drawn_this_turn, 1, "exactly one draw counted");
}

// ── CR 306.9 / 508.4 — combat damage to a planeswalker removes loyalty ────────

fn keyworded_body(name: &'static str, p: i32, t: i32, kws: Vec<Keyword>) -> crabomination::card::CardDefinition {
    crabomination::card::CardDefinition {
        name,
        card_types: vec![crabomination::card::CardType::Creature],
        power: p,
        toughness: t,
        keywords: kws,
        ..Default::default()
    }
}

/// A creature attacking a planeswalker deals its combat damage as loyalty loss
/// (CR 306.9 — that many loyalty counters are removed).
#[test]
fn cr_306_9_combat_damage_to_planeswalker_removes_loyalty() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, keyworded_body("Raider", 3, 3, vec![]));
    g.clear_sickness(atk);
    let pw = g.add_card_to_battlefield(1, catalog::teferi_time_raveler());
    let start = g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty);
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Planeswalker(pw),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers; // no blocks
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    let now = g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty);
    assert_eq!(now, start - 3, "3 combat damage removed 3 loyalty");
}

// ── CR 702.4 / 702.9 — Flying can only be blocked by flying or reach ──────────

/// A flyer can't be blocked by a creature without flying or reach (CR 702.4c).
#[test]
fn cr_702_4c_flyer_cant_be_blocked_by_ground() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, keyworded_body("Drake", 2, 2, vec![Keyword::Flying]));
    g.clear_sickness(atk);
    let ground = g.add_card_to_battlefield(1, keyworded_body("Ogre", 3, 3, vec![]));
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(ground, atk)])).is_err(),
        "a ground creature can't block a flyer",
    );
}

/// A creature with reach can block a flyer (CR 702.9c).
#[test]
fn cr_702_9c_reach_blocks_flyer() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, keyworded_body("Drake", 2, 2, vec![Keyword::Flying]));
    g.clear_sickness(atk);
    let spider = g.add_card_to_battlefield(1, keyworded_body("Spider", 1, 3, vec![Keyword::Reach]));
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(spider, atk)]))
        .expect("reach can block the flyer");
}

// ── CR 702.28 — Shadow (near the new Hellbent shadow-grant, Cutthroat il-Dal) ──

/// A creature with shadow can be blocked only by shadow creatures, and can
/// block only shadow creatures (CR 702.28b/c).
#[test]
fn cr_702_28_shadow_restricts_blocking_both_ways() {
    let mut g = two_player_game();
    let shadow_atk = g.add_card_to_battlefield(0, catalog::soltari_priest()); // shadow
    g.clear_sickness(shadow_atk);
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // no shadow
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: shadow_atk, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(ground, shadow_atk)])).is_err(),
        "a non-shadow creature can't block a shadow attacker"
    );
}

/// A normal (non-shadow) attacker can't be blocked by a shadow creature.
#[test]
fn cr_702_28_normal_attacker_unblockable_by_shadow() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    let shadow_blk = g.add_card_to_battlefield(1, catalog::soltari_priest()); // shadow
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(shadow_blk, atk)])).is_err(),
        "a shadow creature can't block a non-shadow attacker"
    );
}

// ── CR 702.23 — Rampage (Craw Giant) ──────────────────────────────────────────

/// Rampage N: the attacker gets +N/+N for each blocker beyond the first
/// (CR 702.23a). Two blockers on a rampage-2 attacker → one extra → +2/+2.
#[test]
fn cr_702_23_rampage_pumps_per_extra_blocker() {
    let mut g = two_player_game();
    let giant = g.add_card_to_battlefield(0, catalog::craw_giant()); // 6/4 rampage 2
    g.clear_sickness(giant);
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(b1);
    g.clear_sickness(b2);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: giant, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, giant), (b2, giant)])).unwrap();
    let cp = g.computed_permanent(giant).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 6), "rampage 2 with one extra blocker → +2/+2");
}

// ── CR 601.2h — sacrifice as an activation cost (DECK_FEATURES 🟡) ─────────────

/// A "Sacrifice this" activation cost is paid before the ability resolves, so
/// the source is already gone when the effect happens (CR 601.2h / 602.2a).
#[test]
fn cr_601_2h_sac_cost_paid_before_effect_resolves() {
    let mut g = two_player_game();
    let replica = g.add_card_to_battlefield(0, catalog::vulshok_replica());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: replica, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate");
    // The source is already sacrificed while the ability is on the stack.
    assert!(g.battlefield_find(replica).is_none(), "sacrificed as a cost, before resolution");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "the 3 damage still resolves from the stack");
}

// ── CR 701.34 — Proliferate over player resource counters ──────────────────────

/// CR 701.34a — proliferate may add a counter to a *player* who has one. The
/// controller proliferates their own experience counter.
#[test]
fn cr_701_34_proliferate_adds_experience_counter() {
    use crabomination::effect::Effect;
    let mut g = two_player_game();
    g.players[0].experience = 2;
    let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    g.resolve_effect(&Effect::Proliferate, &ctx).unwrap();
    assert_eq!(g.players[0].experience, 3, "proliferated one more experience counter");
    // A player without any experience gets none (nothing to proliferate).
    assert_eq!(g.players[1].experience, 0, "no experience → nothing added");
}

// ── CR 122 / 614.16 — Winding Constrictor boosts experience counters ───────────

/// The player half of Winding Constrictor's additive replacement (CR 614.16)
/// applies to experience counters, like energy: "you get that many plus one".
#[test]
fn cr_614_16_winding_constrictor_boosts_experience() {
    use crabomination::effect::{Effect, Value};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::winding_constrictor());
    let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    g.resolve_effect(&Effect::AddExperience(Value::Const(1)), &ctx).unwrap();
    assert_eq!(g.players[0].experience, 2, "got one experience plus one");
}

/// CR 614.16 / 122 — Winding Constrictor's player half boosts poison counters
/// the controller gets, on both the AddPoison and AddCounter(Player) paths.
#[test]
fn cr_614_16_winding_constrictor_boosts_poison() {
    use crabomination::effect::{Effect, PlayerRef, Selector, Value};
    let mut g = two_player_game();
    // Player 1 controls the Constrictor and is the one gaining poison.
    g.add_card_to_battlefield(1, catalog::winding_constrictor());
    let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    g.resolve_effect(&Effect::AddPoison { who: Selector::Player(PlayerRef::Seat(1)), amount: Value::Const(2) }, &ctx).unwrap();
    assert_eq!(g.players[1].poison_counters, 3, "two poison plus one");
    g.resolve_effect(&Effect::AddCounter {
        what: Selector::Player(PlayerRef::Seat(1)),
        kind: crabomination::card::CounterType::Poison,
        amount: Value::Const(2),
    }, &ctx).unwrap();
    assert_eq!(g.players[1].poison_counters, 6, "AddCounter path also boosts: +3 more");
}

// ── CR 115 — an activated ability can target a spell on the stack ──────────────

/// CR 115.4 / 706 — a "copy target spell you control" activated ability
/// validates a spell-on-the-stack target through `evaluate_requirement_static`.
#[test]
fn cr_115_activated_ability_targets_a_stack_spell() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::izzet_guildmage());
    for c in g.battlefield.iter_mut() { c.summoning_sick = false; }
    g.players[0].life = 20;
    g.players[1].life = 20;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt on stack");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Permanent(bolt)),
        additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("the ability accepts the stack-spell target");
    drain_stack(&mut g);
    let dealt = (20 - g.players[0].life) + (20 - g.players[1].life);
    assert_eq!(dealt, 6, "the copied bolt resolved alongside the original");
}

// ── CR 702.6 — Equip (timing & control) ──────────────────────────────────────

/// CR 702.6d — "Equip is a special action ... any time [the controller] could
/// cast a sorcery." Activating it with a non-empty stack / outside a main phase
/// is illegal.
#[test]
fn cr_702_6d_equip_is_sorcery_speed_only() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let axe = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareBlockers; // not a main phase
    assert!(matches!(
        g.perform_action(GameAction::Equip { equipment: axe, target: bear }),
        Err(GameError::SorcerySpeedOnly)
    ));
}

/// CR 702.6c — the equip target must be a creature the activating player
/// controls. Equipping an opponent's creature is illegal.
#[test]
fn cr_702_6c_equip_target_must_be_your_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let axe = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    assert!(matches!(
        g.perform_action(GameAction::Equip { equipment: axe, target: foe }),
        Err(GameError::InvalidTarget)
    ));
}

/// CR 301.5c — an Equipment can be attached to only one creature at a time;
/// re-equipping moves it off the previous host.
#[test]
fn cr_301_5c_reequip_moves_the_equipment() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let axe = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: axe, target: bear1 }).expect("equip bear1");
    assert_eq!(g.computed_permanent(bear1).unwrap().power, 4, "bear1 buffed");
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: axe, target: bear2 }).expect("re-equip bear2");
    assert_eq!(g.computed_permanent(bear1).unwrap().power, 2, "bear1 no longer equipped");
    assert_eq!(g.computed_permanent(bear2).unwrap().power, 4, "bear2 now equipped");
    assert_eq!(g.battlefield_find(axe).unwrap().attached_to, Some(bear2));
}

/// CR 604.3 — a characteristic-defining ability that reads the battlefield
/// recomputes live (Akiri's power tracks the artifact count).
#[test]
fn cr_604_3_artifact_count_cda_recomputes() {
    let mut g = two_player_game();
    let akiri = g.add_card_to_battlefield(0, catalog::akiri_line_slinger());
    assert_eq!(g.computed_permanent(akiri).unwrap().power, 0);
    g.add_card_to_battlefield(0, catalog::bonesplitter());
    assert_eq!(g.computed_permanent(akiri).unwrap().power, 1, "recomputed after an artifact entered");
}

// ── CR 702.49 + 122.1b — a Ninjutsu'd creature's ETB fires (keyword counter) ──

/// CR 702.49d — the ninja "enters the battlefield" via Ninjutsu, so its
/// enters-with-a-deathtouch-counter clause (CR 122.1b) applies just like a
/// normal ETB (Kappa Tech-Wrecker).
#[test]
fn cr_702_49_ninjutsu_entrant_gets_its_enter_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let kappa = g.add_card_to_hand(0, catalog::kappa_tech_wrecker());
    g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Ninjutsu { ninja: kappa, returning: bear })
        .expect("Ninjutsu on an unblocked attacker");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(kappa).unwrap().keywords.contains(&Keyword::Deathtouch),
        "the Ninjutsu'd Kappa entered with its deathtouch counter"
    );
}

// ── CR 509.1b — Menace requires two or more blockers ─────────────────────────

/// CR 509.1b / 702.111 — a creature with menace (here granted by Nezumi
/// Bladeblesser's "menace while you control an enchantment") can't be blocked by
/// exactly one creature.
#[test]
fn cr_509_1b_menace_rejects_a_lone_blocker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::golden_tail_disciple()); // enchantment
    let nezumi = g.add_card_to_battlefield(0, catalog::nezumi_bladeblesser());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(nezumi);
    assert!(g.computed_permanent(nezumi).unwrap().keywords.contains(&Keyword::Menace));
    g.attacking = vec![Attack { attacker: nezumi, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.active_player_idx = 0;
    let res = g.perform_action(GameAction::DeclareBlockers(vec![(blocker, nezumi)]));
    assert!(res.is_err(), "a single blocker can't block a menacing attacker (CR 509.1b)");
}

// ── CR 702.2e + 122.1b — a deathtouch counter makes combat damage lethal ─────

/// CR 702.2e — any nonzero combat damage from a creature with deathtouch (here
/// from a CR 122.1b deathtouch counter, not a printed keyword) is lethal.
#[test]
fn cr_702_2e_deathtouch_counter_is_lethal_in_combat() {
    let mut g = two_player_game();
    let kappa = g.add_card_to_battlefield(0, catalog::kappa_tech_wrecker());
    g.fire_self_etb_triggers(kappa, 0);
    drain_stack(&mut g);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(kappa);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: kappa,
        target: AttackTarget::Player(1),
    }]))
    .expect("Kappa attacks");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, kappa)]))
        .expect("grizzly blocks");
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(blocker).is_none(), "1 deathtouch damage killed the 2/2");
}

/// CR 122 — an `AnyCounterAdded` trigger fires on the first counter placed on a
/// given permanent each turn and no more (Stalwart Successor, `with_per_subject_cap`).
#[test]
fn cr_122_any_counter_added_first_time_per_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stalwart_successor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bump = |g: &mut GameState, id| {
        g.battlefield_find_mut(id).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        g.dispatch_triggers_for_events(&[GameEvent::CounterAdded {
            card_id: id,
            counter_type: CounterType::PlusOnePlusOne,
            count: 1,
        }]);
        drain_stack(g);
    };
    bump(&mut g, bear);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    bump(&mut g, bear); // second placement same turn → no bonus
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// CR 604.3 — a characteristic-defining ability sets power continuously: Snow
/// Villiers's power equals the number of creatures its controller controls.
#[test]
fn cr_604_3_creature_count_cda() {
    let mut g = two_player_game();
    let snow = g.add_card_to_battlefield(0, catalog::snow_villiers());
    assert_eq!(g.computed_permanent(snow).unwrap().power, 1);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(snow).unwrap().power, 2, "power recomputes as creatures change");
    // An opponent's creature doesn't count.
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(snow).unwrap().power, 2, "only your creatures count");
}

/// CR 702.6e — an Equipment's granted triggered ability fires off the equipped
/// creature (White Mage's Staff grants "attacks → gain 1 life").
#[test]
fn cr_702_6e_equipment_grants_attack_trigger() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::white_mages_staff());
    drain_stack(&mut g);
    let hero = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Hero").unwrap().id;
    g.clear_sickness(hero);
    let life = g.players[0].life;
    while g.step != crabomination::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hero,
        target: AttackTarget::Player(1),
    }])).expect("equipped Hero attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "the granted attack trigger gained 1 life");
}

// ── CR 608.2h — last-known information for a source that left the battlefield ──

/// A "Sacrifice this: deals damage equal to its power" ability uses the source's
/// last-known power, not 0, because the source is already gone when the ability
/// resolves (Blazing Bomb).
#[test]
fn cr_608_2h_sacrificed_source_deals_last_known_power() {
    let mut g = two_player_game();
    let bomb = g.add_card_to_battlefield(0, catalog::blazing_bomb());
    g.clear_sickness(bomb);
    g.battlefield_find_mut(bomb).unwrap().add_counters(CounterType::PlusOnePlusOne, 2); // 3/3
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bomb, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Blow Up");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bomb).is_none(), "source sacrificed as a cost");
    assert!(g.battlefield_find(foe).is_none(), "dealt its last-known power (3) — 2/2 destroyed");
}

// ── CR 702.28e — Landcycling fetches a basic land of the named type ────────────

/// Mountaincycling discards the card and searches for a Mountain (Hill Gigas).
#[test]
fn cr_702_28e_mountaincycling_fetches_a_mountain() {
    let mut g = two_player_game();
    let mtn = g.add_card_to_library(0, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::hill_gigas());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Landcycle { card_id: id }).expect("mountaincycle");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mtn), "Mountain fetched to hand");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Hill Gigas discarded");
}

// ── CR 613.7 — layer 7 P/T: dynamic anthem + counters combine ─────────────────

/// A team anthem (layer 7c ModifyPT) stacks with +1/+1 counters (layer 7d), and
/// the anthem's scaling count reacts live to the board (Warrior of Light).
#[test]
fn cr_613_7_dynamic_anthem_stacks_with_counters() {
    let mut g = two_player_game();
    let wol = g.add_card_to_battlefield(0, catalog::warrior_of_light()); // 5/5 legendary
    // One legendary → anthem +1/+1 → 6/6.
    assert_eq!(g.computed_permanent(wol).unwrap().power, 6);
    // A +1/+1 counter stacks on top of the anthem (7 total power).
    g.battlefield_find_mut(wol).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert_eq!(g.computed_permanent(wol).unwrap().power, 7, "counter (7d) adds to anthem (7c)");
    // A second legendary raises the anthem to +2/+2 live: 5 + 2 + 1 = 8.
    g.add_card_to_battlefield(0, catalog::edgar_king_of_figaro());
    assert_eq!(g.computed_permanent(wol).unwrap().power, 8, "anthem re-scales with the board");
}

// ── CR 701.22 / 701.42 — "whenever you scry or surveil" ────────────────────

/// A scry and a surveil each fire a "whenever you scry or surveil" trigger,
/// but "look at and rearrange the top N" (RearrangeTop — Index / Spire Owl)
/// is neither a scry nor a surveil (CR 701.22 / 701.42) and does not.
#[test]
fn cr_701_22_scry_and_surveil_trigger_but_rearrange_does_not() {
    use crabomination::effect::{Effect, PlayerRef, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let matoya = g.add_card_to_battlefield(0, catalog::matoya_archon_elder());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let run = |g: &mut GameState, eff: Effect| {
        let ctx = EffectContext::for_ability(matoya, 0, None);
        let events = g.resolve_effect(&eff, &ctx).unwrap();
        g.dispatch_triggers_for_events(&events);
        drain_stack(g);
    };
    let before = g.players[0].hand.len();
    run(&mut g, Effect::Scry { who: PlayerRef::You, amount: Value::ONE });
    assert_eq!(g.players[0].hand.len(), before + 1, "scry drew via Matoya");
    run(&mut g, Effect::Surveil { who: PlayerRef::You, amount: Value::ONE });
    assert_eq!(g.players[0].hand.len(), before + 2, "surveil drew via Matoya");
    // RearrangeTop is not a scry/surveil → no draw.
    run(&mut g, Effect::RearrangeTop { who: PlayerRef::You, amount: Value::ONE });
    assert_eq!(g.players[0].hand.len(), before + 2, "rearrange did not trigger Matoya");
}

/// CR 701.22b — a scry 0 is not a scry, so "whenever you scry" triggers stay
/// silent (no draw off Matoya).
#[test]
fn cr_701_22b_scry_zero_does_not_trigger() {
    use crabomination::effect::{Effect, PlayerRef, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let matoya = g.add_card_to_battlefield(0, catalog::matoya_archon_elder());
    g.add_card_to_library(0, catalog::forest());
    let before = g.players[0].hand.len();
    let ctx = EffectContext::for_ability(matoya, 0, None);
    let events = g
        .resolve_effect(&Effect::Scry { who: PlayerRef::You, amount: Value::Const(0) }, &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before, "scry 0 did not trigger");
}

// ── CR 604.3 — characteristic-defining P/T recomputes live ─────────────────

/// Xande, Dark Mage is a 3/3 that grows by +1/+1 for each noncreature,
/// nonland card in its controller's graveyard, recomputed continuously.
#[test]
fn cr_604_3_xande_grows_with_graveyard_cda() {
    let mut g = two_player_game();
    let xande = g.add_card_to_battlefield(0, catalog::xande_dark_mage());
    let cp = g.computed_permanent(xande).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let cp = g.computed_permanent(xande).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "one noncreature card → 4/4");
    // A creature card in the graveyard does not count.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(xande).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "creature card excluded");
}

// ── CR 700.4 / 603.10 — "dies" and last-known info for the new
//    `EventKind::CreatureOrArtifactDied` rail (Judge Magister Gabranth) ────────

/// CR 700.4 — "dies" means *put into a graveyard from the battlefield*. When a
/// death is replaced by exile (Rest in Peace), no `CreatureOrArtifactDied`
/// event fires, so Gabranth doesn't grow.
#[test]
fn cr_700_4_creature_or_artifact_died_not_fired_on_exile_replacement() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rest_in_peace()); // graveyard → exile
    let gabranth = g.add_card_to_battlefield(0, catalog::judge_magister_gabranth());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let evs = g.remove_to_graveyard_with_triggers(ally);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(gabranth).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0,
        "a creature exiled instead of dying is not a death"
    );
}

/// CR 603.10 / 111.7 — a token creature's death is captured before the token
/// ceases to exist, so a "creature or artifact you control dies" watcher still
/// sees it.
#[test]
fn cr_603_10_token_death_still_triggers_creature_or_artifact_died() {
    let mut g = two_player_game();
    let gabranth = g.add_card_to_battlefield(0, catalog::judge_magister_gabranth());
    let token = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(token).unwrap().is_token = true;
    let evs = g.remove_to_graveyard_with_triggers(token);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(gabranth).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "a friendly token dying still grows Gabranth"
    );
}

/// The `CreatureOrArtifactDied` type filter: a non-creature, non-artifact
/// permanent (a land) dying is a death, but neither a creature nor an artifact,
/// so Gabranth ignores it.
#[test]
fn creature_or_artifact_died_ignores_noncreature_nonartifact() {
    let mut g = two_player_game();
    let gabranth = g.add_card_to_battlefield(0, catalog::judge_magister_gabranth());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let evs = g.remove_to_graveyard_with_triggers(land);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(gabranth).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0,
        "a land dying is neither a creature nor an artifact"
    );
}

// ── CR 603.2e — instance-granted combat-damage triggers fire ──────────────────

/// A triggered ability granted by an effect (`Effect::GrantTriggeredAbility`,
/// e.g. a Saga chapter that gives its own creature a keyword ability) fires on
/// the same events as a printed one — here a `DealsCombatDamageToPlayer`
/// self-trigger reaches the combat-damage dispatch (CR 603.2e / 702.6-style
/// grant), not just printed/statics-granted abilities.
#[test]
fn cr_603_2e_granted_combat_damage_trigger_fires() {
    use crabomination::card::{CardDefinition, CardType, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crabomination::effect::{Duration, Effect, Selector, Value};
    let granter = CardDefinition {
        name: "Grantest",
        cost: crabomination::mana::cost(&[crabomination::mana::g()]),
        card_types: vec![CardType::Creature],
        power: 3,
        toughness: 3,
        triggered_abilities: vec![crabomination::effect::shortcut::etb(Effect::GrantTriggeredAbility {
            what: Selector::This,
            trigger: Box::new(TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            }),
            duration: Duration::Permanent,
        })],
        ..Default::default()
    };
    let mut g = two_player_game();
    let atk = g.move_card_to_battlefield_for_test(0, granter);
    drain_stack(&mut g); // ETB installs the granted trigger
    g.clear_sickness(atk);
    let life = g.players[0].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g); // the granted trigger resolves off the stack
    assert_eq!(g.players[0].life, life + 3, "granted combat-damage trigger gained life");
}

// ── CR 702.19a — trample is inert while the creature is blocking ──────────────

/// Trample only modifies an *attacking* creature's damage assignment; a
/// trampler that's blocking assigns all its damage to the attacker and no
/// "excess" reaches any player (CR 702.19a).
#[test]
fn cr_702_19a_trample_inert_while_blocking() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    // Seat 1 (active) attacks with a 3/3 into seat 0's 5/5 trample blocker.
    let attacker = g.add_card_to_battlefield(1, kw_creature("Runner", 3, 3, &[]));
    let blocker = g.add_card_to_battlefield(0, kw_creature("Wall", 5, 5, &[Keyword::Trample]));
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]))
        .expect("block");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(attacker).is_none(), "3/3 died to the 5/5");
    assert!(g.battlefield_find(blocker).is_some(), "5/5 blocker survived 3 damage");
    assert_eq!(g.players[0].life, life0, "trampling blocker deals no excess to any player");
    assert_eq!(g.players[1].life, life1, "attacker's controller takes nothing");
}

// ── CR 700.4 — a noncreature artifact dying triggers "creature or artifact died" ─

/// "Whenever a creature or artifact you control dies" fires off a *noncreature*
/// artifact's death, backed by the single battlefield→graveyard chokepoint
/// (CR 700.4).
#[test]
fn cr_700_4_noncreature_artifact_death_triggers_payoff() {
    let mut g = two_player_game();
    let gabranth = g.add_card_to_battlefield(0, catalog::judge_magister_gabranth());
    let relic = g.add_card_to_battlefield(0, catalog::sol_ring()); // a noncreature artifact
    assert!(!g.battlefield_find(relic).unwrap().definition.is_creature());
    let evs = g.remove_to_graveyard_with_triggers(relic);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(gabranth).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Gabranth grew when the noncreature artifact died",
    );
}

/// CR 120.3 / 104.3c — attempting to draw from an empty library (via a draw
/// *effect*, not just the draw step) loses the game, recorded as `Decked`.
#[test]
fn cr_120_3_overdraw_effect_decks_the_player() {
    use crabomination::effect::{Effect, PlayerRef, Selector, Value};
    use crabomination::player::LossCause;
    let mut g = two_player_game();
    // Exactly one card in P0's library; draw two → the second draw is illegal.
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::island());
    let eff = Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::Const(2) };
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.players[0].eliminated, "drawing from empty library loses");
    assert_eq!(g.players[0].loss_cause, Some(LossCause::Decked), "cause recorded as Decked");
}

/// CR 514.2 × 500.7 — an "until end of turn" pump expires at the single cleanup
/// that follows a *looped* end step (Y'shtola's extra end step), not partway
/// through, and not never.
#[test]
fn cr_514_2_eot_pump_expires_after_extra_end_step() {
    use crabomination::effect::{Duration, Selector, Value};
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eff = Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(3),
        toughness: Value::Const(0),
        duration: Duration::EndOfTurn,
    };
    let ctx = EffectContext::for_ability(bear, 0, None);
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "EOT pump applied");
    // Reach the end step and bank an extra one.
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.additional_end_steps = 1;
    // Walk into the next turn (past both end steps and the one cleanup).
    let start_turn = g.turn_number;
    while g.turn_number == start_turn {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "pump expired at cleanup");
}

/// CR 702.166 — Corrupted: Bonepicker Skirge gains deathtouch and lifelink only
/// while an opponent has three or more poison counters.
#[test]
fn cr_702_166_corrupted_gates_static_keywords() {
    let mut g = two_player_game();
    let skirge = g.add_card_to_battlefield(0, catalog::bonepicker_skirge());
    // No poison yet → neither keyword is live.
    let kw = g.computed_permanent(skirge).unwrap().keywords;
    assert!(!kw.contains(&Keyword::Deathtouch), "no deathtouch below 3 poison");
    assert!(!kw.contains(&Keyword::Lifelink), "no lifelink below 3 poison");
    // Opponent reaches 3 poison → Corrupted turns on.
    g.players[1].poison_counters = 3;
    let kw = g.computed_permanent(skirge).unwrap().keywords;
    assert!(kw.contains(&Keyword::Deathtouch), "deathtouch live at 3 poison");
    assert!(kw.contains(&Keyword::Lifelink), "lifelink live at 3 poison");
    // Two poison is below the threshold.
    g.players[1].poison_counters = 2;
    assert!(
        !g.computed_permanent(skirge).unwrap().keywords.contains(&Keyword::Deathtouch),
        "threshold is strictly three",
    );
}

// ── CR 702.11e — "hexproof from nongreen" targeting gate (Thrun) ─────────────

/// An opponent's nongreen spell can't target a creature with "can't be the
/// target of nongreen spells opponents control"; a green spell can.
#[test]
fn cr_702_11e_hexproof_from_nongreen_blocks_only_nongreen_opponents() {
    let mut g = two_player_game();
    let thrun = g.add_card_to_battlefield(0, catalog::thrun_breaker_of_silence());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    // Opponent's red Lightning Bolt is rejected.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(thrun)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "nongreen opponent spell can't target Thrun");
    // Opponent's green Giant Growth is allowed (green is exempt).
    let gg = g.add_card_to_hand(1, catalog::giant_growth());
    g.players[1].mana_pool.add(Color::Green, 1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: gg, target: Some(Target::Permanent(thrun)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_ok(), "a green opponent spell may target Thrun");
}

// ── CR 104.3c / 704.5c — ten poison counters is a game loss ──────────────────

/// A player with ten or more poison counters loses when SBAs are checked (the
/// end-state of the toxic/infect payoffs).
#[test]
fn cr_704_5c_ten_poison_counters_loses() {
    let mut g = two_player_game();
    assert!(!g.players[1].eliminated);
    g.players[1].poison_counters = 10;
    g.check_state_based_actions();
    assert!(g.players[1].eliminated, "ten poison counters loses the game (CR 704.5c)");
}

// ── CR 702.180c — Toxic adds poison equal to its value on combat damage ──────

/// A toxic-4 attacker dealing combat damage to a player gives four poison
/// counters (Tyrranax Rex).
#[test]
fn cr_702_180c_toxic_gives_poison_equal_to_value() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let rex = g.add_card_to_battlefield(0, catalog::tyrranax_rex());
    g.clear_sickness(rex);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rex, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 4, "toxic 4 → four poison counters (CR 702.180c)");
}

/// CR 704.5z — controlling a "Start your engines!" permanent with no speed
/// sets speed 1 via state-based action (covers non-cast arrivals).
#[test]
fn cr_704_5z_engines_seed_speed_sba() {
    let mut g = two_player_game();
    let mut racer = catalog::grizzly_bears();
    racer.keywords.push(crabomination::card::Keyword::StartYourEngines);
    g.add_card_to_battlefield(0, racer);
    assert_eq!(g.players[0].speed, 0, "SBA hasn't run yet");
    g.check_state_based_actions();
    assert_eq!(g.players[0].speed, 1, "704.5z seeded speed 1");
    g.check_state_based_actions();
    assert_eq!(g.players[0].speed, 1, "idempotent");
}

/// CR 702.65 — Aura swap exchanges the Aura with one in hand, keeping the host.
#[test]
fn cr_702_65_aura_swap_exchanges_with_hand() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wings = g.add_card_to_battlefield(0, catalog::arcanum_wings());
    g.battlefield_find_mut(wings).unwrap().attached_to = Some(bears);
    let other = g.add_card_to_hand(0, catalog::zealots_conviction());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wings, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("aura swap");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == wings), "Wings back in hand");
    let conviction = g.battlefield_find(other).expect("hand Aura deployed");
    assert_eq!(conviction.attached_to, Some(bears), "attached to the same host");
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "Conviction's +1/+1 applies");
}

// ── CR 500.9 — additional upkeep steps ───────────────────────────────────────

/// Paradox Haze banks an additional upkeep step at the first upkeep of your
/// turn; the extra upkeep begins (and doesn't loop a third time).
#[test]
fn cr_500_9_paradox_haze_grants_second_upkeep() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::paradox_haze());
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.upkeep_steps_this_turn = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.additional_upkeep_steps, 1, "haze banked an extra upkeep");
    // Pass until the banked step loops back into a second Upkeep.
    while g.upkeep_steps_this_turn == 1 && !g.is_game_over() {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.upkeep_steps_this_turn, 2, "second upkeep step");
    assert_eq!(g.step, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.additional_upkeep_steps, 0, "the extra upkeep doesn't re-trigger the haze");
    // And the turn proceeds to Draw afterwards.
    while g.step == TurnStep::Upkeep && !g.is_game_over() {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.step, TurnStep::Draw, "no third upkeep");
}

// ── CR 702.62e — suspend's eventual cast respects cast-zone locks ────────────

/// Drannith Magistrate ("opponents can't cast from anywhere but their hands")
/// blocks the suspended card's free cast when the last time counter comes off;
/// it stays exiled (CR 702.62e).
#[test]
fn cr_702_62e_suspend_final_cast_blocked_by_drannith() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::durkwood_baloth()); // Suspend 5—{G}
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Suspend { card_id: id }).expect("suspend");
    g.add_card_to_battlefield(1, catalog::drannith_magistrate());
    g.exile.iter_mut().find(|c| c.id == id).unwrap().counters
        .insert(CounterType::Time, 1);
    g.active_player_idx = 0;
    let evs = g.process_suspend();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let card = g.exile.iter().find(|c| c.id == id).expect("stays exiled while locked");
    assert_eq!(card.counter_count(CounterType::Time), 0, "counters still ticked away");
    assert!(g.battlefield_find(id).is_none(), "never entered the battlefield");
}

// ── CR 702.137 — Adamant (per-color mana-spent tracking) ────────────────────

/// Slaying Fire deals 4 with three red spent, 3 on a mixed payment.
#[test]
fn cr_702_137_adamant_reads_colored_payment() {
    let mut g = two_player_game();
    let fire = g.add_card_to_hand(0, catalog::slaying_fire());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: fire, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 4, "RRR payment: adamant 4");

    let fire2 = g.add_card_to_hand(0, catalog::slaying_fire());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: fire2, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 7, "GRR payment: plain 3");
}

// ── CR 601 — Void Mirror's "no colored mana spent" gate ──────────────────────

/// Void Mirror counters an all-generic-paid spell but not a colored one.
#[test]
fn cr_601_void_mirror_counters_colorless_casts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::void_mirror());
    // A colorless artifact paid with colorless mana: countered.
    let bone = g.add_card_to_hand(1, catalog::batterbone());
    g.players[1].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bone, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bone).is_none(), "countered — no colored mana spent");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bone));
    // A creature paid with colored mana resolves.
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "colored payment resolves");
}

// ── CR 702.14c — filtered landwalk (artifact landwalk) ───────────────────────

/// Vectis Gloves grants artifact landwalk: unblockable only while the
/// defending player controls an artifact land.
#[test]
fn cr_702_14c_artifact_landwalk() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gloves = g.add_card_to_battlefield(0, catalog::vectis_gloves());
    g.battlefield.iter_mut().find(|c| c.id == gloves).unwrap().attached_to = Some(bear);
    g.clear_sickness(bear);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _citadel = g.add_card_to_battlefield(1, catalog::darksteel_citadel());
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, bear)])).is_err(),
        "artifact landwalk: unblockable while they control Darksteel Citadel"
    );
    // Without the artifact land the block is legal.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gloves = g.add_card_to_battlefield(0, catalog::vectis_gloves());
    g.battlefield.iter_mut().find(|c| c.id == gloves).unwrap().attached_to = Some(bear);
    g.clear_sickness(bear);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    let r = g.perform_action(GameAction::DeclareBlockers(vec![(blocker, bear)]));
    assert!(r.is_ok(), "block should be legal without an artifact land: {r:?}");
}

// ── CR 903.4 — color identity includes indicators + ability costs ────────────

/// A costless card with a red color indicator and a {W} activated ability has
/// a white-red identity.
#[test]
fn cr_903_4_identity_reads_indicator_and_abilities() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::effect::Effect;
    let def = CardDefinition {
        name: "Identity Probe",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Red],
        activated_abilities: vec![crabomination::card::ActivatedAbility {
            mana_cost: crabomination::mana::cost(&[crabomination::mana::w()]),
            effect: Effect::Noop,
            ..Default::default()
        }],
        ..Default::default()
    };
    let id = crabomination::format::color_identity(&def);
    assert!(id.contains(Color::Red), "color indicator counts (CR 903.4)");
    assert!(id.contains(Color::White), "activated-ability cost counts");
    assert!(!id.contains(Color::Green));
}

/// CR 704.5q — an Equipment attached to a creature that leaves the battlefield
/// becomes unattached (as a state-based action) but stays on the battlefield.
#[test]
fn cr_704_5q_equipment_unattaches_when_host_dies() {
    let mut g = two_player_game();
    let sword = g.move_card_to_battlefield_for_test(0, catalog::warriors_sword());
    drain_stack(&mut g);
    let hero = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Hero").unwrap().id;
    assert_eq!(g.battlefield_find(sword).unwrap().attached_to, Some(hero), "attached at first");
    let evs = g.remove_to_graveyard_with_triggers(hero);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    // The stale-link SBA runs when a player would get priority.
    g.check_state_based_actions();
    let s = g.battlefield_find(sword).expect("Equipment stays on the battlefield");
    assert_eq!(s.attached_to, None, "Equipment became unattached when its host left");
}

/// CR 707.2e — a token that's a copy of a permanent copies its printed
/// characteristics, not the counters on it. The Fire Crystal copies a bear with
/// a +1/+1 counter; the token is a printed 2/2, not 3/3.
#[test]
fn cr_707_2e_token_copy_ignores_source_counters() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_fire_crystal());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "the source bear is a 3/3 with its counter");
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    let crystal = g.battlefield.iter().find(|c| c.definition.name == "The Fire Crystal").unwrap().id;
    g.perform_action(GameAction::ActivateAbility {
        card_id: crystal, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("copy the bear");
    drain_stack(&mut g);
    let token = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Grizzly Bears")
        .expect("token copy exists");
    let tp = g.computed_permanent(token.id).unwrap();
    assert_eq!((tp.power, tp.toughness), (2, 2), "the copy uses printed P/T, ignoring the source's counter");
}

// ── CR 704.5g — Aura-granted indestructible survives lethal damage ───────────

/// A creature made indestructible by an Aura (layer-6 grant — Shielded by
/// Faith) is not destroyed by lethal marked damage (CR 704.5g). Regression for
/// the SBA reading *computed* indestructible, not just the printed keyword.
#[test]
fn cr_704_5g_aura_granted_indestructible_survives_lethal() {
    use crabomination::game::types::Target;
    use crabomination::game::{drain_stack, GameAction};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::shielded_by_faith());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Shielded by Faith");
    drain_stack(&mut g);
    g.battlefield_find_mut(bear).unwrap().damage = 10; // way past lethal
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_some(), "indestructible creature survives lethal damage");
}

// ── CR 613 — Opalescence-animated enchantment dies to lethal damage ──────────

/// Opalescence makes Nevermore ({1}{W}{W}, MV 3) a 3/3 creature; three marked
/// damage is then lethal and the SBA destroys it (layer-7 P/T feeds 704.5g).
#[test]
fn cr_613_opalescence_animated_enchantment_dies_to_lethal() {
    use crabomination::card::CardType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::opalescence());
    let nm = g.add_card_to_battlefield(0, catalog::nevermore());
    let cp = g.computed_permanent(nm).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature) && cp.toughness == 3);
    g.battlefield_find_mut(nm).unwrap().damage = 3; // lethal for the 3/3
    g.check_state_based_actions();
    assert!(g.battlefield_find(nm).is_none(), "the 3/3 animated enchantment dies");
}

// ── CR 702.16e — protection from creatures blocks a creature blocker ─────────

/// A creature with protection from creatures (Unquestioned Authority) can't be
/// blocked by a creature (CR 509.1b / 702.16e).
#[test]
fn cr_702_16e_protection_from_creatures_cant_be_blocked() {
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    use crabomination::game::{drain_stack, GameAction};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::unquestioned_authority());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.add_card_to_library(0, catalog::forest());
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(attacker)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Unquestioned Authority");
    drain_stack(&mut g);
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut guard = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
        guard += 1;
        assert!(guard < 60);
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).is_err(),
        "a creature can't block a creature with protection from creatures"
    );
}

// ── CR 701.47c — Amass loads counters onto an existing Army ───────────────────

/// CR 701.47c — amass puts counters on an Army you already control instead of
/// minting a second one. Two Mindless Conscription ETBs (amass Zombies 3 each)
/// leave a single 6/6 Army.
#[test]
fn cr_701_47c_amass_grows_existing_army() {
    let mut g = two_player_game();
    for _ in 0..2 {
        let id = g.add_card_to_hand(0, catalog::mindless_conscription());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
    }
    let armies: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Army").collect();
    assert_eq!(armies.len(), 1, "only one Army — the second amass grew the first");
    assert_eq!(armies[0].counter_count(CounterType::PlusOnePlusOne), 6, "3 + 3 counters");
}

// ── CR 702.114 — Devoid ──────────────────────────────────────────────────────

/// CR 702.114 — a Devoid card is colorless even though its cost has colored
/// pips. Hope-Ender Coatl is {2}{U} with devoid.
#[test]
fn cr_702_114_devoid_is_colorless() {
    assert!(catalog::hope_ender_coatl().printed_colors().is_empty(), "devoid printed colors empty");
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::hope_ender_coatl());
    assert!(g.computed_permanent(id).unwrap().colors.is_empty(), "colorless on the battlefield");
}

// ── CR 702.137 — Adapt ───────────────────────────────────────────────────────

/// CR 702.137b — Adapt N does nothing if the creature already has a +1/+1
/// counter. A second activation of Dreamdrinker Vampire's adapt is a no-op.
#[test]
fn cr_702_137_adapt_noop_when_already_has_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::dreamdrinker_vampire());
    g.clear_sickness(id);
    for _ in 0..2 {
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("adapt");
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "second adapt added nothing — already had a counter");
}

// ── CR 107.14 — the energy symbol {E} ────────────────────────────────────────

/// CR 107.14 — to pay {E} a player removes one energy counter. Aether Spike's
/// `PayAnyEnergy` (bot: pay all) removes exactly the counters it spends, and
/// scales the counter-tax by that count. Starting 3 + the spell's {E}{E} = 5.
#[test]
fn cr_107_14_pay_any_energy_removes_exactly_paid_counters() {
    let mut g = two_player_game();
    g.players[0].energy = 3;
    // P1 casts a creature spending all mana (can't afford the {5} tax).
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a spell");
    let id = g.add_card_to_hand(0, catalog::aether_spike());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(spell)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Aether Spike");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 0, "removed all 5 energy counters to pay {{E}} five times");
    assert!(g.battlefield_find(spell).is_none(), "spell countered — {{5}} tax unpaid");
}

// ── CR 614 — as-enters (replacement) applies before the first SBA ────────────

/// CR 614 — Corrupted Shapeshifter's "as it enters, it becomes your choice"
/// is a replacement, not an ETB trigger: the chosen P/T is in place before the
/// first state-based-action check, so a printed */* (0/0) never dies as a 0/0.
#[test]
fn cr_614_enters_as_choice_precedes_sba() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::corrupted_shapeshifter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Corrupted Shapeshifter");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).expect("survived the ETB SBA — not a dead 0/0");
    assert!(cp.toughness > 0, "entered with a chosen positive toughness");
}

// ── CR 118/119/120 — "life total can't change" vs. combat damage ─────────────

/// CR 119.3/120.3 — combat damage to a player causes life loss, but a player
/// whose life total can't change this turn (Flare of Fortitude) loses none:
/// the damage is dealt, the reduction is dropped at the life chokepoint.
#[test]
fn cr_119_life_lock_prevents_combat_damage_loss() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.players[0].life_locked_this_turn = true;
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(0) }];
    g.step = TurnStep::CombatDamage;
    g.active_player_idx = 1;
    let before = g.players[0].life;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before, "locked life total unchanged by the 2 combat damage");
}

// ── CR 702.92 — Battle cry ───────────────────────────────────────────────────

/// CR 702.92a — "Whenever this creature attacks, each other attacking creature
/// gets +1/+0 until end of turn." Granted mid-turn (Reckless Pyrosurfer's
/// landfall) it still fires from the whole-batch attack view and skips itself.
#[test]
fn cr_702_92_battle_cry_pumps_only_other_attackers() {
    let mut g = two_player_game();
    let surfer = g.add_card_to_battlefield(0, catalog::reckless_pyrosurfer());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(ally);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.declare_attackers(vec![
        Attack { attacker: surfer, target: AttackTarget::Player(1) },
        Attack { attacker: ally, target: AttackTarget::Player(1) },
    ]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ally).unwrap().power, 3, "other attacker +1/+0");
    assert_eq!(g.computed_permanent(surfer).unwrap().power, 2, "battle cry excludes its source");
}

// ── CR 122.1b — keyword counters ─────────────────────────────────────────────

/// CR 122.1b — a keyword counter grants its keyword. A Saga chapter that puts a
/// vigilance counter on a creature (Ajani Fells the Godsire II) makes that
/// creature vigilant via the counter, not a granted-keyword static.
#[test]
fn cr_122_1b_keyword_counter_grants_keyword() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::ajani_fells_the_godsire());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Chapter I auto-targets nothing meaningful without an enemy, so seed one.
    g.add_card_to_battlefield(1, catalog::serra_angel());
    g.saga_advance(saga); // I
    drain_stack(&mut g);
    g.saga_advance(saga); // II — vigilance counter on a creature you control
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ally).unwrap().keyword_counters
        .get(&crabomination::card::Keyword::Vigilance).copied().unwrap_or(0), 1,
        "a vigilance counter is on the creature");
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&crabomination::card::Keyword::Vigilance),
        "the keyword counter grants vigilance");
}

// ── CR 712 — modal double-faced cards ────────────────────────────────────────

/// CR 712.9/712.14 — a modal DFC's back land face is chosen when the card is
/// played; it enters and functions as that face only (Boggart Bog, the pain-
/// land back of Boggart Trawler, taps for B).
#[test]
fn cr_712_modal_dfc_back_is_the_played_face() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::boggart_trawler());
    g.perform_action(GameAction::PlayLandBack(id)).expect("play the land back");
    drain_stack(&mut g);
    let land = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(land.definition.name, "Boggart Bog");
    assert!(land.definition.is_land(), "entered as its land face");
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("taps for {B}");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1, "back face taps for B");
}

// ── CR 117.7c / 601.2f — cost reductions touch only the generic component ─────

/// CR 117.7c — a "costs {N} less" reduction removes only generic mana; the
/// colored requirement survives. Deem Inferior ({3}{U}) after four draws
/// reduces its {3} to nothing and is castable for a single {U}.
#[test]
fn cr_117_7c_cost_reduction_is_generic_only() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.players[0].cards_drawn_this_turn = 4; // reduces {3} fully (clamped at 3)
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1); // only the {U}
    let deem = g.add_card_to_hand(0, catalog::deem_inferior());
    g.perform_action(GameAction::CastSpell {
        card_id: deem, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for just {U} — the generic {3} is fully reduced");
}

// ── CR 702.90 — Exalted stacks once per instance ─────────────────────────────

/// CR 702.90b — each Exalted instance triggers separately when a creature
/// attacks alone. Two Exalted sources pump the lone attacker +2/+2.
#[test]
fn cr_702_90_exalted_stacks_per_instance() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::ignoble_hierarch());
    g.add_card_to_battlefield(0, catalog::ignoble_hierarch());
    g.clear_sickness(a);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("advance");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
    ])).expect("attack alone");
    drain_stack(&mut g);
    let p = g.computed_permanent(a).unwrap();
    assert_eq!((p.power, p.toughness), (2, 3), "0/1 + two exalted instances = 2/3");
}

// ── CR 401.7 — Nth-from-top falls back to the bottom of a short library ───────

/// CR 401.7 — putting a card "second from the top" of a library with fewer
/// than two cards puts it on the bottom instead. Deem Inferior into an empty
/// owner library leaves the tucked permanent as the sole (bottom) card.
#[test]
fn cr_401_7_second_from_top_falls_to_bottom_when_library_short() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.players[1].library.clear();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // "second from top"
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let deem = g.add_card_to_hand(0, catalog::deem_inferior());
    g.perform_action(GameAction::CastSpell {
        card_id: deem, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), 1, "the tucked permanent is the only card");
    assert_eq!(g.players[1].library[0].id, victim, "it went to the bottom");
}

// ── CR 115.1c — an "up to N target" self-cast trigger maximizes its targets ───

/// CR 115.1c — Twisted Riddlekeeper's "when you cast this spell, tap up to two
/// target permanents" self-cast trigger auto-fills *both* slots (the engine
/// maximizes an engine-resolved "up to N" trigger the same way it does an ETB).
#[test]
fn cr_115_1c_cast_trigger_maximizes_up_to_two_targets() {
    let mut g = two_player_game();
    g.step = crabomination::game::TurnStep::PreCombatMain;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::twisted_riddlekeeper());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 10);
    }
    g.players[0].mana_pool.add_colorless(10);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("hardcast");
    drain_stack(&mut g);
    for id in [a, b] {
        assert!(g.battlefield_find(id).unwrap().tapped, "both up-to-two targets tapped");
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Stun), 1);
    }
}

// ── CR 702.119 — Emerge reduces the cost by the sacrificed creature's MV ──────

/// CR 702.119c — casting Twisted Riddlekeeper for Emerge {5}{C}{U} while
/// sacrificing a mana-value-5 creature reduces the generic {5} to {0}: the
/// caster pays only {C}{U} and the fodder is sacrificed.
#[test]
fn cr_702_119_emerge_reduces_generic_by_sacrificed_mana_value() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::serra_angel()); // MV 5
    let id = g.add_card_to_hand(0, catalog::twisted_riddlekeeper());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("emerge cast for {C}{U} after MV-5 sacrifice");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == fodder), "fodder sacrificed");
    let cp = g.computed_permanent(id).expect("Riddlekeeper resolved");
    assert_eq!((cp.power, cp.toughness), (5, 5), "printed 5/5");
}

// ── CR 702.114 — Devoid makes a card colorless regardless of its mana cost ────

/// CR 702.114 — Thief of Existence is cast for {1}{C}{G} yet, being Devoid, has
/// no color on the battlefield.
#[test]
fn cr_702_114_devoid_creature_is_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::thief_of_existence());
    assert!(g.computed_permanent(id).unwrap().colors.is_empty(), "Devoid → colorless");
}

// ── CR 107.16 — {E} (energy) as a variable activation cost ───────────────────

/// CR 107.16 — an ability whose cost includes "Pay X {E}" spends exactly the
/// chosen X, and the same X gates the target (Chthonian Nightmare reanimates a
/// creature card whose mana value equals the energy paid).
#[test]
fn cr_107_16_variable_energy_cost_pays_chosen_x() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let nightmare = g.add_card_to_battlefield(0, catalog::chthonian_nightmare());
    g.players[0].energy = 6;
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    g.perform_action(GameAction::ActivateAbility {
        card_id: nightmare, ability_index: 0,
        target: Some(Target::Permanent(dead)),
        additional_targets: vec![], x_value: Some(2), mode: None,
    }).expect("pay 2 {E} for a MV-2 reanimation");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 4, "spent exactly the chosen X = 2 energy");
    assert!(g.battlefield.iter().any(|c| c.id == dead), "MV-2 target reanimated");
}

// ── CR 508 — observer "whenever [a player] attacks" triggers ─────────────────

/// CR 508.1 — an `AnyPlayer`-scoped "whenever two or more creatures attack"
/// trigger (Argent Dais) fires for its controller even when the *opponent* is
/// the one declaring the attackers.
#[test]
fn cr_508_observer_attack_trigger_sees_opponents_attack() {
    let mut g = two_player_game();
    // Dais controlled by P0; the opponent (P1) attacks with two creatures.
    let dais = g.add_card_to_battlefield(0, catalog::argent_dais());
    let before = g.battlefield_find(dais).unwrap().counter_count(CounterType::Oil);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(0) },
    ]).expect("opponent declares two attackers");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(dais).unwrap().counter_count(CounterType::Oil), before + 1,
        "P0's Dais gains oil from P1's multi-creature attack");
}

// ── CR 122.1 — "enters with N counters" ──────────────────────────────────────

/// CR 122.1 — a permanent printed to enter with counters (Argent Dais, two oil)
/// arrives already bearing them.
#[test]
fn cr_122_1_permanent_enters_with_printed_counters() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let id = g.add_card_to_hand(0, catalog::argent_dais());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Argent Dais");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Oil), 2,
        "enters with two oil counters");
}

// ── CR 603.2 — Aura "when enchanted creature is dealt damage" trigger ─────────

/// CR 603.2 / 303.4 — an Aura whose triggered ability watches its enchanted
/// creature fires on a *damage* event, not just death (Cracked Skull destroys
/// the creature it enchants the moment that creature is dealt damage).
#[test]
fn cr_603_2_enchanted_creature_dealt_damage_trigger() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let skull = g.add_card_to_hand(0, catalog::cracked_skull());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: skull, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant the bear");
    drain_stack(&mut g);
    let mut ev = vec![];
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(bear), 1, None, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "damage triggers the destroy");
}

// ── CR 601.2f — Delirium cost reduction is generic-only ──────────────────────

/// CR 601.2f — a card-intrinsic "{2} less" Delirium discount reduces only the
/// generic part of the cost; the colored pips still have to be paid. Drag to
/// the Roots ({2}{B}{G}) casts for {B}{G} under Delirium but not for less.
#[test]
fn cr_601_2f_delirium_reduction_is_generic_only() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    // Four card types in the graveyard → Delirium.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(0, catalog::divination());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::drag_to_the_roots());
    // Only the two colored pips are available — the {2} generic is fully discounted.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("casts for just {B}{G} under Delirium");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "the discounted spell resolved");
}

// ── CR 701.20 — "put on the top or bottom of library" (owner's choice) ────────

/// CR 701.20 — a "puts it on top or bottom, owner's choice" tuck defaults to the
/// bottom under the AutoDecider (Dire Downdraft), and the reduction only applies
/// against a tapped/attacking target.
#[test]
fn cr_701_20_owner_choice_tuck_defaults_bottom() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::dire_downdraft());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dire Downdraft");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(bear), "tucked to the bottom");
}

// ── CR 702.171 — Saddle N (the special "tap other creatures" ability) ────────

/// CR 702.171 — Saddle N requires tapping other untapped creatures with total
/// power N or more; the Mount is saddled only until end of turn (702.171e).
#[test]
fn cr_702_171_saddle_requires_power_and_clears_at_end_of_turn() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let gryff = g.add_card_to_battlefield(0, catalog::congregation_gryff()); // Saddle 3
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // One 2/2 (power 2 < 3) can't pay the saddle cost.
    let rejected = g.perform_action(GameAction::Saddle { mount: gryff, creatures: vec![bear] });
    assert!(matches!(rejected, Err(GameError::SelectionRequirementViolated)), "power 2 < Saddle 3");
    assert!(!g.battlefield_find(gryff).unwrap().saddled, "not saddled after a rejected attempt");
    // Two 2/2s (total power 4 ≥ 3) can.
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear2);
    g.perform_action(GameAction::Saddle { mount: gryff, creatures: vec![bear, bear2] })
        .expect("saddle with total power 4");
    assert!(g.battlefield_find(gryff).unwrap().saddled, "Mount saddled");
    assert!(g.battlefield_find(bear).unwrap().tapped, "rider tapped as the cost");
    // Walk into the next turn; cleanup clears the "until end of turn" saddle.
    let start = g.turn_number;
    while g.turn_number == start {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(!g.battlefield_find(gryff).unwrap().saddled, "saddled cleared at end of turn (702.171e)");
}

// ── CR 700.13 — committing a crime (targeting an opponent's stuff) ───────────

/// CR 700.13 — a spell that targets a permanent an opponent controls commits a
/// crime, which fires "whenever you commit a crime" payoffs (Blood Hustler).
#[test]
fn cr_700_13_targeting_opponent_permanent_is_a_crime() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let hustler = g.add_card_to_battlefield(0, catalog::blood_hustler());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Lightning Bolt at the opponent's creature");
    drain_stack(&mut g);
    assert!(g.players[0].committed_crime_this_turn, "targeting an opponent's permanent is a crime");
    assert_eq!(g.computed_permanent(hustler).unwrap().power, 2, "Blood Hustler grew from the crime");
}

// ── CR 604.3 — characteristic-defining ability P/T recomputes continuously ───

/// CR 604.3 — Duelist of the Mind's `*` power is a CDA equal to cards drawn this
/// turn; it recomputes the instant another card is drawn, in every zone check.
#[test]
fn cr_604_3_cda_power_tracks_cards_drawn_live() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let duelist = g.add_card_to_battlefield(0, catalog::duelist_of_the_mind());
    let mut events = vec![];
    assert_eq!(g.computed_permanent(duelist).unwrap().power, 0, "0 drawn → 0 power");
    g.draw_one(0, &mut events);
    assert_eq!(g.computed_permanent(duelist).unwrap().power, 1, "recomputes to 1 after a draw");
    g.draw_one(0, &mut events);
    g.draw_one(0, &mut events);
    assert_eq!(g.computed_permanent(duelist).unwrap().power, 3, "recomputes to 3 after two more");
}

/// CR 604.3 — a `*/*` characteristic-defining P/T recomputes live as nonland
/// permanents enter and leave (Regal Bunnicorn).
#[test]
fn cr_604_3_nonland_permanent_cda_recomputes() {
    let mut g = two_player_game();
    let bunny = g.add_card_to_battlefield(0, catalog::regal_bunnicorn());
    assert_eq!(g.computed_permanent(bunny).unwrap().power, 1, "alone → 1");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::forest()); // land doesn't count
    assert_eq!(g.computed_permanent(bunny).unwrap().power, 2, "2 nonland permanents");
    g.remove_from_battlefield_to_graveyard_raw(bear);
    assert_eq!(g.computed_permanent(bunny).unwrap().power, 1, "recomputes when one leaves");
}

/// CR 509.1b — the fixed-threshold "can't be blocked by creatures with power N
/// or greater" restriction (Squeak By's granted keyword) is the mirror of
/// Questing Beast's "power N or less".
#[test]
fn cr_509_1b_power_at_least_block_restriction() {
    use crabomination::card::{CardDefinition, CardType, Keyword};
    let mut g = two_player_game();
    let evader = g.add_card_to_battlefield(0, CardDefinition {
        name: "Test Evader",
        card_types: vec![CardType::Creature],
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::CantBeBlockedByPowerAtLeast(3)],
        ..Default::default()
    });
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    assert!(g.blocker_can_block_attacker(small, evader), "power-2 can block");
    assert!(!g.blocker_can_block_attacker(big, evader), "power-4 can't block");
}

/// CR 613.7d — switching power and toughness is applied last (layer 7d), after
/// +1/+1 counters (layer 7c): a base 2/4 with a +1/+1 counter (3/5) switched
/// becomes 5/3, not 4/3.
#[test]
fn cr_613_7d_switch_pt_applies_after_counters() {
    use crabomination::card::{CardDefinition, CardType, CounterType};
    use crabomination::effect::{Duration, Effect, Selector};
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, CardDefinition {
        name: "Test Body",
        card_types: vec![CardType::Creature],
        power: 2,
        toughness: 4,
        ..Default::default()
    });
    g.battlefield_find_mut(c).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert_eq!((g.computed_permanent(c).unwrap().power, g.computed_permanent(c).unwrap().toughness), (3, 5));
    let ctx = crabomination::game::effects::EffectContext::for_ability(c, 0, Some(Target::Permanent(c)));
    g.resolve_effect(
        &Effect::SwitchPT { what: Selector::Target(0), duration: Duration::EndOfTurn },
        &ctx,
    )
    .unwrap();
    let cp = g.computed_permanent(c).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 3), "switch happens after the counter");
}

/// CR 700.4 — "dies" means put into a graveyard from the battlefield. A
/// non-creature permanent's own `PermanentDied`/SelfSource trigger (a Wicked
/// Role token's death-drain) fires when it is sacrificed, not only creatures'.
#[test]
fn cr_700_4_noncreature_self_death_trigger_fires() {
    use crabomination::card::{CardDefinition, CardType, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crabomination::effect::{Effect, PlayerRef, Selector, Value};
    let mut g = two_player_game();
    let glyph = g.add_card_to_battlefield(0, CardDefinition {
        name: "Wicked Glyph",
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    });
    g.players[1].life = 20;
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        glyph, 0, Some(crabomination::game::types::Target::Permanent(glyph)),
    );
    g.resolve_effect(
        &Effect::SacrificePermanent { what: Selector::Target(0) },
        &ctx,
    )
    .unwrap();
    g.dispatch_triggers_for_events(&[]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "the enchantment's own death trigger drained the opponent");
}

// ── CR 704.5y — a creature keeps only the newest Role its controller owns ─────

/// CR 704.5y — if a permanent has two or more Role Auras controlled by the same
/// player, the older ones are put into the graveyard as a state-based action.
#[test]
fn cr_704_5y_second_role_replaces_first() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Monstrous Rage grants a Monster Role; a second Role must evict the first.
    let rage1 = g.add_card_to_hand(0, catalog::monstrous_rage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: rage1,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    // A second Monster Role must evict the first — a creature keeps one Role.
    let rage2 = g.add_card_to_hand(0, catalog::monstrous_rage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: rage2,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    let roles = g.battlefield.iter().filter(|c| c.attached_to == Some(bear)
        && c.definition.name == "Monster").count();
    assert_eq!(roles, 1, "only the newest Role survives (CR 704.5y)");
}

// ── CR 603.4 / 506.5 — an Attacks intervening-if reads the attacker's P/T ─────

/// CR 603.4 + 506.5 — a "whenever this attacks, if its toughness is 3 or less"
/// trigger is evaluated post-batch against the attacker's live toughness, so a
/// pump above the threshold before combat suppresses it.
#[test]
fn cr_603_4_attacker_toughness_intervening_if() {
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::types::{AttackTarget, Target};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    // Young Hero Role via Embereth Veteran's sac.
    let vet = g.add_card_to_battlefield(0, catalog::embereth_veteran());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: vet,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .unwrap();
    drain_stack(&mut g);
    // Pump the bear's toughness above 3 before it attacks.
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        bear, 0, Some(Target::Permanent(bear)),
    );
    g.resolve_effect(
        &Effect::PumpPT {
            what: Selector::Target(0),
            power: Value::Const(0),
            toughness: Value::Const(3),
            duration: crabomination::effect::Duration::EndOfTurn,
        },
        &ctx,
    )
    .unwrap();
    g.clear_sickness(bear);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        0,
        "toughness 5 > 3 suppresses the Young Hero counter (CR 603.4)",
    );
}

// ── CR 715.3 — an adventure card exiles after its adventure resolves ─────────

/// CR 715.3 — casting the Adventure half exiles the card afterwards, from where
/// its creature half can be cast; `SpellCast.adventuring` marks the cast.
#[test]
fn cr_715_3_adventure_card_exiles_for_later_creature_cast() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Puny Snack (Gingerbread Hunter's adventure): -2/-2.
    let gh = g.add_card_to_hand(0, catalog::gingerbread_hunter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastAdventure {
        card_id: gh,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert!(
        g.exile.iter().any(|c| c.id == gh),
        "the adventure card sits in exile, ready to be cast as its creature (CR 715.3)",
    );
}

// ── CR 702.166 — Bargain (optional additional sacrifice cost) ─────────────────

/// Bargain lets a spell's controller sacrifice an artifact, enchantment, or
/// token as an additional cost; `SpellWasBargained` then gates the bonus
/// (Torch the Tower: 3 damage instead of 2).
#[test]
fn cr_702_166_bargain_sacrifice_enables_bonus() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let target = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let token = g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
    let id = g.add_card_to_hand(0, catalog::torch_the_tower());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpellBargain {
        card_id: id,
        sacrifice: Some(token),
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(token).is_none(), "the token was sacrificed to bargain");
    assert_eq!(g.battlefield_find(target).unwrap().damage, 3, "bargained Torch deals 3, not 2");
}

// ── CR 701.21 — "whenever you sacrifice a [permanent]" watcher ────────────────

/// Sacrificing a permanent (here a Food, to its own ability) fires a
/// `PermanentSacrificed` watcher (Experimental Confectioner → Rat).
#[test]
fn cr_701_21_sacrifice_a_food_fires_watcher() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::experimental_confectioner());
    let food = g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: food,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Rat"),
        "sacrificing a Food fired the PermanentSacrificed watcher (CR 701.21)",
    );
}

// ── CR 702.19b — basic trample assigns lethal, then excess to the player ──────

/// A plain (single-strike) trampler assigns exactly lethal to its lone blocker
/// and tramples the remainder to the defending player in the combat step.
#[test]
fn cr_702_19b_trample_overflows_single_blocker() {
    fn big_trampler() -> crabomination::card::CardDefinition {
        crabomination::card::CardDefinition {
            name: "Test Trampler",
            cost: crabomination::mana::cost(&[crabomination::mana::generic(4), crabomination::mana::g()]),
            card_types: vec![crabomination::card::CardType::Creature],
            power: 5,
            toughness: 5,
            keywords: vec![crabomination::card::Keyword::Trample],
            ..Default::default()
        }
    }
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, big_trampler());
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(blocker);
    let life = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert!(g.battlefield_find(blocker).is_none(), "2 lethal to the 2/2 blocker");
    assert_eq!(g.players[1].life, life - 3, "5 - 2 lethal = 3 tramples to the player");
}

/// CR 509.1a — a tapped creature can't be declared as a blocker.
#[test]
fn cr_509_1a_tapped_creature_cannot_block() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(blocker).unwrap().tapped = true;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    let res = g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]));
    assert!(matches!(res, Err(GameError::CannotBlock(_))), "tapped creature rejected as blocker");
}

/// CR 702.19e — a trampler with deathtouch needs assign only 1 (lethal) to each
/// blocker before the rest tramples through.
#[test]
fn cr_702_19e_deathtouch_trample_assigns_one() {
    fn deadly_trampler() -> crabomination::card::CardDefinition {
        crabomination::card::CardDefinition {
            name: "Deadly Trampler",
            cost: crabomination::mana::cost(&[crabomination::mana::generic(4), crabomination::mana::g()]),
            card_types: vec![crabomination::card::CardType::Creature],
            power: 5,
            toughness: 5,
            keywords: vec![crabomination::card::Keyword::Trample, crabomination::card::Keyword::Deathtouch],
            ..Default::default()
        }
    }
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, deadly_trampler());
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.clear_sickness(blocker);
    let life = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert!(g.battlefield_find(blocker).is_none(), "deathtouch kills the 4/4 with 1 damage");
    assert_eq!(g.players[1].life, life - 4, "only 1 lethal assigned; 4 tramples over");
}

/// CR 603.3d — a "triggers only once each turn" ability fires once even when
/// its event happens twice in a turn (Sharae's tap-draw).
#[test]
fn cr_603_3d_once_each_turn_fires_once() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sharae_of_numbing_depths());
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: enemy, actor: Some(0), as_attacker: false }]);
    drain_stack(&mut g);
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: enemy, actor: Some(0), as_attacker: false }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "only one draw despite two taps this turn");
}

/// CR 502.3 — an Aura's "doesn't untap during its controller's untap step"
/// lock is continuous, not sticky: once the Aura leaves the battlefield the
/// creature untaps normally again.
#[test]
fn cr_502_3_untap_lock_releases_when_aura_leaves() {
    let mut g = two_player_game();
    let chill = g.add_card_to_battlefield(0, catalog::bitter_chill());
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(chill).unwrap().attached_to = Some(creature);
    g.battlefield_find_mut(creature).unwrap().tapped = true;
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(creature).unwrap().tapped, "locked while enchanted");
    // Remove the Aura; the lock lifts.
    g.battlefield.retain(|c| c.id != chill);
    g.do_untap();
    assert!(!g.battlefield_find(creature).unwrap().tapped, "untaps once the Aura is gone");
}

/// CR 613.7 — a state-set base P/T (layer 7b) composes with a +1/+1 counter
/// (layer 7c): Archon of the Wild Rose sets enchanted creatures to base 4/4,
/// and a counter stacks on top for 5/5.
#[test]
fn cr_613_7_set_base_pt_stacks_with_counter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::archon_of_the_wild_rose());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::pacifism());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5), "base 4/4 (7b) + counter (7c) = 5/5");
}

/// CR 401.6 / 603.3d — Johann's once-each-turn top-of-library cast refreshes
/// at the turn boundary.
#[test]
fn cr_401_6_johann_cap_resets_next_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::johann_apprentice_sorcerer());
    let bolt1 = g.add_card_to_library(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_library(0, catalog::lightning_bolt());
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("first top cast");
    drain_stack(&mut g);
    assert!(g.players[0].cast_from_library_top_this_turn, "cap consumed");
    // The next untap step clears the charge (do_untap turn-boundary reset).
    g.active_player_idx = 0;
    g.do_untap();
    assert!(!g.players[0].cast_from_library_top_this_turn, "charge reset at untap");
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("second top cast after reset");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "both Bolts resolved across two turns");
}

/// CR 509.1b/702.111 — Menace: a creature with menace can't be blocked by
/// fewer than two creatures. One blocker is illegal; two is legal.
#[test]
fn cr_509_1b_menace_requires_two_blockers() {
    use crabomination::card::{CardDefinition, CardType, Keyword};
    use crabomination::game::types::Attack;
    let menacer_def = CardDefinition {
        name: "Menacer",
        card_types: vec![CardType::Creature],
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        ..Default::default()
    };
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, menacer_def);
    g.clear_sickness(attacker);
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareBlockers;
    g.active_player_idx = 0;
    assert!(g.declare_blockers(vec![(b1, attacker)]).is_err(), "one blocker is illegal");
    g.declare_blockers(vec![(b1, attacker), (b2, attacker)]).expect("two blockers is legal");
}

// ── CR 603.12 — reflexive "when you do" triggers only on the action ───────────
/// A reflexive triggered ability fires only when its gating action is actually
/// taken during the parent's resolution: Bloodcrazed Socialite's on-attack
/// "you may sacrifice a Blood; when you do, it gets +2/+2" pumps only when a
/// Blood is present to sacrifice — with no Blood the payoff never fires.
#[test]
fn cr_603_12_reflexive_payoff_gated_on_the_action() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let attack = |g: &mut GameState, id| {
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: id,
            target: AttackTarget::Player(1),
        }]))
        .unwrap();
        drain_stack(g);
    };
    // No Blood to sacrifice → reflexive payoff never fires; stays 3/3.
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::bloodcrazed_socialite());
    g.clear_sickness(s);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, s);
    assert_eq!(
        g.computed_permanent(s).map(|c| (c.power, c.toughness)),
        Some((3, 3)),
        "no Blood → the reflexive +2/+2 does not fire (CR 603.12)"
    );
    // With a Blood present and accepted, the sacrifice happens → +2/+2.
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::bloodcrazed_socialite());
    g.clear_sickness(s);
    g.add_token_to_battlefield(0, &crabomination_base::tokens::blood_token());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, s);
    assert_eq!(
        g.computed_permanent(s).map(|c| (c.power, c.toughness)),
        Some((5, 5)),
        "sacrificing the Blood fires the reflexive +2/+2 (CR 603.12)"
    );
}

// ── CR 606.3 — a loyalty ability: once per turn, only at sorcery speed ────────
/// A planeswalker's loyalty ability may be activated only when its controller
/// could cast a sorcery, and only once per permanent per turn.
#[test]
fn cr_606_3_loyalty_once_per_turn_and_sorcery_speed() {
    let mut g = two_player_game();
    let nissa = g.add_card_to_battlefield(0, catalog::nissa_voice_of_zendikar());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    // First +1 (make a Plant): loyalty 3 → 4.
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: nissa, ability_index: 0, target: None, x_value: None,
    })
    .expect("first loyalty activation succeeds");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(nissa).unwrap().counter_count(CounterType::Loyalty), 4);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Plant" && c.controller == 0));
    // Second activation the same turn → rejected (CR 606.3).
    let err = g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: nissa, ability_index: 0, target: None, x_value: None,
    })
    .unwrap_err();
    assert!(matches!(err, GameError::LoyaltyAbilityAlreadyUsed(id) if id == nissa));
    // Sorcery-speed gate: on the opponent's turn it can't be activated at all.
    let mut g = two_player_game();
    let nissa = g.add_card_to_battlefield(0, catalog::nissa_voice_of_zendikar());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    let err = g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: nissa, ability_index: 0, target: None, x_value: None,
    })
    .unwrap_err();
    assert!(matches!(err, GameError::SorcerySpeedOnly), "off-turn activation is illegal (CR 606.3)");
}

// ── CR 702.2b — deathtouch: any damage from a deathtouch source is lethal ─────
/// A 1/1 deathtouch blocker destroys a 5/5 attacker as a state-based action:
/// one point of deathtouch combat damage is lethal (CR 702.2b).
#[test]
fn cr_702_2b_deathtouch_one_damage_is_lethal() {
    fn tiny_deathtoucher() -> crabomination::card::CardDefinition {
        crabomination::card::CardDefinition {
            name: "Tiny Deathtoucher",
            card_types: vec![crabomination::card::CardType::Creature],
            power: 1,
            toughness: 1,
            keywords: vec![crabomination::card::Keyword::Deathtouch],
            ..Default::default()
        }
    }
    fn bruiser() -> crabomination::card::CardDefinition {
        crabomination::card::CardDefinition {
            name: "Bruiser",
            card_types: vec![crabomination::card::CardType::Creature],
            power: 5,
            toughness: 5,
            ..Default::default()
        }
    }
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, bruiser());
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, tiny_deathtoucher());
    g.clear_sickness(blocker);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert!(g.battlefield_find(attacker).is_none(), "1 deathtouch damage kills the 5/5 (CR 702.2b)");
    assert!(g.battlefield_find(blocker).is_none(), "the 1/1 also dies to 5 damage");
}

// ── CR 702.21 — Ward with a fixed life cost ──────────────────────────────────
/// A creature with "Ward—Pay N life" (a fixed `WardCost::Life(n)`, distinct from
/// the power-scaled variant) counters an opponent's targeting spell unless they
/// pay exactly N life. Sire of Seven Deaths — Ward—Pay 7 life.
#[test]
fn cr_702_21_fixed_ward_life_is_paid() {
    let mut g = two_player_game();
    let sire = g.add_card_to_battlefield(0, catalog::sire_of_seven_deaths()); // 7/7, Ward—Pay 7
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    let before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(sire)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast bolt at the warded creature");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 7, "paid the fixed Ward—7 life");
    assert!(g.battlefield_find(sire).is_some(), "7/7 shrugs off the 3-damage bolt");
}

/// CR 702.21 + 701.59 — "Ward—Collect evidence N" counters an opponent's
/// targeting spell unless they exile cards with total mana value ≥ N from their
/// graveyard. Axebane Ferox — Ward—Collect evidence 4.
#[test]
fn cr_701_59_ward_collect_evidence_paid_or_countered() {
    // Enough graveyard fuel: the ward is paid (evidence exiled) and the bolt
    // resolves, marking damage on the 4/4.
    let mut g = two_player_game();
    let ferox = g.add_card_to_battlefield(0, catalog::axebane_ferox());
    for _ in 0..2 { g.add_card_to_graveyard(1, catalog::grizzly_bears()); } // MV 2 × 2 = 4
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(ferox)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt at the warded creature");
    drain_stack(&mut g);
    assert_eq!(g.exile.iter().filter(|c| c.owner == 1).count(), 2, "evidence exiled to pay ward");
    assert_eq!(g.battlefield_find(ferox).unwrap().damage, 3, "bolt resolved after ward paid");

    // Empty graveyard: the ward can't be paid, so the bolt is countered and no
    // damage is marked.
    let mut g = two_player_game();
    let ferox = g.add_card_to_battlefield(0, catalog::axebane_ferox());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(ferox)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ferox).unwrap().damage, 0, "bolt countered by unpaid ward");
}

// ── CR 502.1 — a "doesn't untap during your untap step" self-static ──────────
/// A permanent with `StaticEffect::PreventUntap { This }` stays tapped through
/// its controller's untap step every turn (not a one-shot lock). Slumbering
/// Cerberus.
#[test]
fn cr_502_1_self_static_prevents_untap() {
    let mut g = two_player_game();
    let dog = g.add_card_to_battlefield(0, catalog::slumbering_cerberus());
    g.battlefield_find_mut(dog).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(dog).unwrap().tapped, "stays tapped on the untap step");
    g.do_untap();
    assert!(g.battlefield_find(dog).unwrap().tapped, "still tapped — the static is permanent, not one-shot");
}

// ── CR 509.1b — a granted "can't block" keyword rejects the blocker ──────────
/// A creature granted `Keyword::CantBlock` (Sower of Chaos's activated ability)
/// can't be declared as a blocker.
#[test]
fn cr_509_1b_cant_block_keyword_rejects_blocker() {
    let mut g = two_player_game();
    let sower = g.add_card_to_battlefield(0, catalog::sower_of_chaos());
    g.clear_sickness(sower);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Sower grants the opposing blocker "can't block".
    g.players[0].mana_pool.add(Color::Red, 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sower, ability_index: 0,
        target: Some(Target::Permanent(foe)), additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("grant can't-block");
    drain_stack(&mut g);
    // Sower attacks; the disabled creature may not block it.
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sower, target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    assert!(g.declare_blockers(vec![(foe, sower)]).is_err(), "a can't-block creature can't be declared as a blocker");
}

// ── CR 702.171 — Saddle (sorcery-speed + "other creatures") ─────────────────

/// CR 702.171d — Saddle can be activated only as a sorcery, and CR 702.171a's
/// "other untapped creatures" excludes the Mount itself.
#[test]
fn cr_702_171_saddle_is_sorcery_speed_and_excludes_the_mount() {
    let mut g = two_player_game();
    let mount = g.add_card_to_battlefield(0, catalog::gloryheath_lynx()); // Saddle 2
    let ox = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    g.clear_sickness(mount);
    g.clear_sickness(ox);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // Instant speed (opponent's turn) → rejected.
    g.active_player_idx = 1;
    assert!(
        matches!(
            g.perform_action(GameAction::Saddle { mount, creatures: vec![ox] }),
            Err(GameError::SorcerySpeedOnly)
        ),
        "saddle only as a sorcery",
    );
    // Back on your turn, the Mount can't be one of its own saddlers.
    g.active_player_idx = 0;
    assert!(
        g.perform_action(GameAction::Saddle { mount, creatures: vec![mount] }).is_err(),
        "a Mount can't saddle itself",
    );
    // A legal 2-power saddler works.
    g.perform_action(GameAction::Saddle { mount, creatures: vec![ox] }).expect("saddle");
    assert!(g.battlefield_find(mount).unwrap().saddled, "Mount is saddled");
}

// ── CR 613.1d / 613.4 — additive type-changing (animated Vehicle) ────────────

/// CR 613.4 layer-4: adding the creature type to a Vehicle is *additive* — the
/// permanent keeps its Artifact type, becoming an "artifact creature".
#[test]
fn cr_613_4_animated_vehicle_keeps_artifact_type() {
    let mut g = two_player_game();
    let sub = g.add_card_to_battlefield(0, catalog::invasion_submersible());
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sub, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("exhaust animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(sub).unwrap();
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature), "now a creature");
    assert!(cp.card_types.contains(&crabomination::card::CardType::Artifact), "still an artifact (additive)");
}

// ── CR 701.42 — Surveil ──────────────────────────────────────────────────────

/// CR 701.42a — a surveiled card the player declines to keep on top goes to the
/// graveyard (not the bottom of the library, as scry would).
#[test]
fn cr_701_42_surveil_routes_declined_card_to_graveyard() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::{Effect, PlayerRef, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::grim_bauble());
    let top = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![top],
    }]));
    let ctx = EffectContext::for_ability(src, 0, None);
    let events = g
        .resolve_effect(&Effect::Surveil { who: PlayerRef::You, amount: Value::ONE }, &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "declined surveil card hits the graveyard");
}

// ── CR 702.111 — Menace (block requires two or more creatures) ────────────────

/// A menace attacker (Insatiable Skittermaw) can't be blocked by a single
/// creature, but two blockers is legal.
#[test]
fn cr_702_111_menace_requires_two_blockers() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::insatiable_skittermaw());
    g.clear_sickness(atk);
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareBlockers;
    g.active_player_idx = 0;
    assert!(g.declare_blockers(vec![(b1, atk)]).is_err(), "one blocker rejected by menace");
    g.declare_blockers(vec![(b1, atk), (b2, atk)]).expect("two blockers is legal");
}

// ── CR 702.15 — Lifelink applies to noncombat (ability) damage ────────────────

/// Merciless Enforcers has lifelink; its {3}{B} ping deals 1 damage AND gains
/// its controller 1 life (lifelink is not combat-restricted).
#[test]
fn cr_702_15_lifelink_on_ability_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::merciless_enforcers());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1, "1 damage to opponent");
    assert_eq!(g.players[0].life, life_before + 1, "lifelink gained 1 life");
}

// ── CR 601.2f — target-conditional cost reduction ─────────────────────────────

/// Grow Extra Arms ({1}{G}) costs {1} less when it targets a Spider, so a lone
/// {G} is enough to pump a Spider but not a non-Spider.
#[test]
fn cr_601_2f_cost_reduction_when_targeting_spider() {
    use crabomination::game::types::Target;
    // Targeting a Spider: {G} covers the reduced cost.
    let mut g = two_player_game();
    let spider = g.add_card_to_battlefield(0, catalog::radioactive_spider());
    let cast = g.add_card_to_hand(0, catalog::grow_extra_arms());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.cast_spell(cast, Some(Target::Permanent(spider)), vec![], None, None)
        .expect("{G} pays the Spider-reduced cost");

    // Targeting a non-Spider: {G} alone is short of {1}{G}.
    let mut g2 = two_player_game();
    let bear = g2.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cast2 = g2.add_card_to_hand(0, catalog::grow_extra_arms());
    g2.players[0].mana_pool.add(Color::Green, 1);
    g2.priority.player_with_priority = 0;
    g2.step = TurnStep::PreCombatMain;
    assert!(
        g2.cast_spell(cast2, Some(Target::Permanent(bear)), vec![], None, None).is_err(),
        "no reduction on a non-Spider target — {{G}} is insufficient",
    );
}

// ── CR 301.7 — Vehicles are creatures only when an effect makes them one ──────

/// A Vehicle with a conditional "is an artifact creature" static (Midnight
/// Mangler, `StaticEffect::SelfIsCreatureIf`) counts as a creature only while
/// its predicate holds — here, only during turns that aren't its controller's.
#[test]
fn cr_301_7_conditional_vehicle_is_creature_only_off_turn() {
    use crabomination::card::CardType;
    let mut g = two_player_game();
    let mangler = g.add_card_to_battlefield(0, catalog::midnight_mangler());
    g.active_player_idx = 0;
    assert!(!g.computed_permanent(mangler).unwrap().card_types.contains(&CardType::Creature),
        "not a creature on its controller's turn");
    g.active_player_idx = 1;
    assert!(g.computed_permanent(mangler).unwrap().card_types.contains(&CardType::Creature),
        "an artifact creature during other players' turns");
}

// ── CR 702.122 — Crew (crewing with toughness) ───────────────────────────────

/// A creature that "crews using its toughness rather than its power"
/// (Interface Ace, `StaticEffect::SelfCrewsSaddlesWithToughness`) can online a
/// Crew 2 Vehicle off its toughness even with power 0.
#[test]
fn cr_702_122_crew_counts_toughness_for_interface_ace() {
    let mut g = two_player_game();
    let vehicle = g.add_card_to_battlefield(0, catalog::boommobile()); // Crew 2
    let ace = g.add_card_to_battlefield(0, catalog::interface_ace()); // 0/4
    g.clear_sickness(ace);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Crew { vehicle, crew_creatures: vec![ace] })
        .expect("toughness 4 ≥ Crew 2 even though power is 0");
}

// ── CR 702.171 — Saddle (Effect::SetSaddled) ─────────────────────────────────

/// `Effect::SetSaddled` (Guidelight Matrix) marks a target Mount saddled
/// without tapping riders, and the marker clears at end of turn (702.171b).
#[test]
fn cr_702_171_effect_set_saddled_marks_mount() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let matrix = g.add_card_to_battlefield(0, catalog::guidelight_matrix());
    let mount = g.add_card_to_battlefield(0, catalog::bridled_bighorn());
    g.clear_sickness(matrix);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: matrix, ability_index: 0, target: Some(Target::Permanent(mount)),
        additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("saddle the mount");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mount).unwrap().saddled, "Mount marked saddled");
}

/// CR 701.9 — a discard that removes several cards in one resolution fires a
/// single "discard one or more cards" batch carrying the count (Magmakin
/// Artillerist deals that much to each opponent, once).
#[test]
fn cr_701_9_discard_batch_fires_once_with_count() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::magmakin_artillerist());
    for _ in 0..3 { g.add_card_to_hand(0, catalog::forest()); }
    let opp = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(
        &crabomination::effect::Effect::Discard {
            who: crabomination::effect::Selector::You,
            amount: crabomination::effect::Value::Const(3),
            random: false,
        },
        &ctx,
    ).unwrap();
    // Exactly one batch event for the discarder, carrying the full count.
    let batches: Vec<_> = events.iter().filter(|e| matches!(e,
        crabomination::game::types::GameEvent::DiscardedBatch { player: 0, .. })).collect();
    assert_eq!(batches.len(), 1, "one batch per resolution");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 3, "3 discarded → 3 damage, applied once");
}

/// CR 514.3 / 701.9 — the cleanup discard-down is still "discard one or more
/// cards", so it fires the batch trigger once for the count (Magmakin bolts
/// the opponent for the two cards trimmed at cleanup).
#[test]
fn cr_514_3_cleanup_discard_fires_batch_trigger() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::magmakin_artillerist());
    // Nine cards in hand → discard two down to the seven-card maximum.
    for _ in 0..9 { g.add_card_to_hand(0, catalog::forest()); }
    let opp = g.players[1].life;
    g.active_player_idx = 0;
    let mut events = Vec::new();
    g.do_cleanup(&mut events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 7, "trimmed to the maximum hand size");
    assert_eq!(g.players[1].life, opp - 2, "cleanup discard of two bolts for two, once");
}

/// CR 702.179 — "each player who doesn't have max speed" excludes speed-4
/// players (Outpace Oblivion's sacrifice).
#[test]
fn cr_702_179_each_player_without_max_speed_excludes_speed_4() {
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(0, catalog::outpace_oblivion());
    g.players[0].speed = 4;
    g.players[1].speed = 0; // no speed also counts as below max
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ench, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0, "max-speed player spared");
    assert_eq!(g.players[1].life, l1 - 2, "no-speed player took 2");
}

/// CR 115.1c — a self-source Attacks trigger with an "up to N target" effect
/// fills every slot, not just the first (Lagorin counters two Vehicles).
#[test]
fn cr_115_1c_attack_trigger_fills_all_target_slots() {
    let mut g = two_player_game();
    let lagorin = g.add_card_to_battlefield(0, catalog::lagorin_soul_of_alacria());
    let v1 = g.add_card_to_battlefield(0, catalog::skybox_ferry());
    let v2 = g.add_card_to_battlefield(0, catalog::veloheart_bike());
    g.clear_sickness(lagorin);
    g.battlefield_find_mut(lagorin).unwrap().saddled = true;
    cr_advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lagorin, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(v1).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(v2).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// CR 115.1c — an engine-resolved "up to N target" **ETB** ability maximizes its
/// targets: Azorius Justiciar detains *two* opposing creatures, not one.
#[test]
fn cr_115_1c_etb_trigger_fills_all_target_slots() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let jus = g.add_card_to_hand(0, catalog::azorius_justiciar());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: jus, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().detained_by.is_some(), "first detained");
    assert!(g.battlefield_find(b).unwrap().detained_by.is_some(), "second detained");
}

/// CR 601.2b — a permanent's ETB *triggered* ability reads the cast's X. Casting
/// Dune Drifter for X=4 lets its ETB return an MV-4 card from the graveyard;
/// before this run the trigger evaluated X as 0.
#[test]
fn cr_601_2b_etb_trigger_reads_cast_x() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::hill_giant()); // {3}{R} = MV 4
    let spell = g.add_card_to_hand(0, catalog::dune_drifter());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4); // X=4
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: Some(4),
    }).expect("cast Dune Drifter X=4");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == dead), "MV-4 card returned by X=4 ETB");
}

/// CR 121.2a — a draw-count replacement can be gated on a condition. Vnwxt's
/// "max speed — draw two instead" doubles only while its controller is at speed 4.
#[test]
fn cr_121_2a_conditional_draw_replacement() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vnwxt_verbose_host());
    for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let mut events = Vec::new();
    g.players[0].speed = 2;
    let n = g.players[0].hand.len();
    g.draw_one(0, &mut events);
    assert_eq!(g.players[0].hand.len(), n + 1, "below max: single draw");
    g.players[0].speed = 4;
    let n = g.players[0].hand.len();
    g.draw_one(0, &mut events);
    assert_eq!(g.players[0].hand.len(), n + 2, "max speed: doubled draw");
}

/// CR 702.179f — a player with no speed has speed 0 for effects that refer to
/// speed. The Speed Demon's end step draws/loses X=0 before any engine starts.
#[test]
fn cr_702_179f_no_speed_counts_as_zero() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_speed_demon());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].speed = 0; // never started engines
    g.active_player_idx = 0;
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "speed 0 → no life lost");
    assert_eq!(g.players[0].hand.len(), hand, "speed 0 → no cards drawn");
}

// ── CR 603.10 — Last-known information for a died creature's controller ──────

/// A graveyard-scoped "when a creature you control dies" trigger reads the
/// dead creature's last-known controller (CR 603.10), so it fires only for
/// your creatures — never an opponent's. Furious Forebear.
#[test]
fn cr_603_10_died_creature_controller_read_from_lki() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forebear = g.add_card_to_graveyard(0, catalog::furious_forebear());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    // An opponent's creature dying must NOT return Forebear.
    let snap_t = g.battlefield_find(theirs).unwrap().clone();
    let mut evs = g.remove_to_graveyard_with_triggers(theirs);
    g.died_card_snapshots.insert(theirs, snap_t);
    evs.push(GameEvent::CreatureDied { card_id: theirs });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == forebear),
        "opponent's creature dying leaves Forebear in the graveyard"
    );
    // Your own creature dying returns it.
    let snap_m = g.battlefield_find(mine).unwrap().clone();
    let mut evs = g.remove_to_graveyard_with_triggers(mine);
    g.died_card_snapshots.insert(mine, snap_m);
    evs.push(GameEvent::CreatureDied { card_id: mine });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.id == forebear),
        "your creature dying returns Forebear to hand"
    );
}

// ── CR 107.16 — variable "Pay X life" activation cost ───────────────────────

/// An activated ability whose cost includes "Pay X life" (CR 107.16) drains
/// exactly the chosen X, and the same X flows into resolution. Krumar Initiate.
#[test]
fn cr_107_16_pay_x_life_variable_activation_cost() {
    let mut g = two_player_game();
    let init = g.add_card_to_battlefield(0, catalog::krumar_initiate());
    g.clear_sickness(init);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: init,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: Some(3), mode: None,
    })
    .expect("endure 3 for X=3 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 3, "paid exactly X=3 life");
    // Endure 3 grew the 2/2 by X.
    assert!(g.computed_permanent(init).unwrap().power >= 5, "endured X=3");
}

// ── CR 702.169 — Mobilize tokens are tapped, attacking, and transient ───────

/// A Mobilize attack trigger mints a tapped-and-attacking Warrior that is
/// sacrificed at end of combat (CR 702.169). Zurgo's Vanguard.
#[test]
fn cr_702_169_mobilize_token_is_tapped_attacking_and_sacrificed() {
    let mut g = two_player_game();
    let zurgo = g.add_card_to_battlefield(0, catalog::zurgos_vanguard());
    g.clear_sickness(zurgo);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: zurgo,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let warrior = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.name == "Warrior")
        .expect("minted a Warrior");
    assert!(warrior.tapped, "token is tapped");
    assert!(g.attacking.iter().any(|a| a.attacker == warrior.id), "token is attacking");
    // End of combat cleanup sacrifices it.
    g.process_attacking_token_cleanup();
    assert!(
        !g.battlefield.iter().any(|c| c.definition.name == "Warrior"),
        "Warrior sacrificed at end of combat"
    );
}

/// CR 702.93 — Undying: a creature that dies with no +1/+1 counter returns with
/// one; a creature that already had a +1/+1 counter stays dead.
#[test]
fn cr_702_93_undying_returns_only_without_counter() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = two_player_game();
    let ghoul = g.add_card_to_battlefield(0, kw_creature("Ghoul", 2, 2, &[Keyword::Undying]));
    g.battlefield_find_mut(ghoul).unwrap().damage = 2; // lethal
    g.check_state_based_actions();
    drain_stack(&mut g);
    let back = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Ghoul")
        .expect("returned via Undying");
    assert_eq!(back.counter_count(CounterType::PlusOnePlusOne), 1, "returns with a +1/+1 counter");

    let ghoul2 = g.add_card_to_battlefield(0, kw_creature("Ghoul2", 2, 2, &[Keyword::Undying]));
    g.battlefield_find_mut(ghoul2).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
    g.battlefield_find_mut(ghoul2).unwrap().damage = 3; // lethal (3/3 with counter)
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == ghoul2),
        "already had a +1/+1 counter → stays in the graveyard",
    );
}

/// CR 702.180 — Toxic N adds N poison counters when a creature deals combat
/// damage to a player, on top of the normal life loss.
#[test]
fn cr_702_180_toxic_adds_poison_on_combat_damage() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, kw_creature("Stinger", 2, 2, &[Keyword::Toxic(2)]));
    g.clear_sickness(atk);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).unwrap();
    g.step = TurnStep::CombatDamage;
    let life = g.players[1].life;
    g.resolve_combat().unwrap();
    assert_eq!(g.players[1].poison_counters, 2, "Toxic 2 → 2 poison");
    assert_eq!(g.players[1].life, life - 2, "and still deals its 2 combat damage");
}

/// CR 702.71 — Wither: combat damage to a creature is dealt as -1/-1 counters,
/// but combat damage to a player is normal life loss (no poison, unlike Infect).
#[test]
fn cr_702_71_wither_creatures_as_counters_players_as_life() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = two_player_game();
    let blocked = g.add_card_to_battlefield(0, kw_creature("Rotfang", 3, 3, &[Keyword::Wither]));
    let unblocked = g.add_card_to_battlefield(0, kw_creature("Rotmaw", 3, 3, &[Keyword::Wither]));
    g.clear_sickness(blocked);
    g.clear_sickness(unblocked);
    let wall = g.add_card_to_battlefield(1, kw_creature("Wall", 0, 5, &[]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: blocked, target: AttackTarget::Player(1) },
        Attack { attacker: unblocked, target: AttackTarget::Player(1) },
    ]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, blocked)])).unwrap();
    g.step = TurnStep::CombatDamage;
    let life = g.players[1].life;
    g.resolve_combat().unwrap();
    assert_eq!(
        g.battlefield_find(wall).unwrap().counter_count(CounterType::MinusOneMinusOne),
        3,
        "wither → -1/-1 counters on the blocking creature",
    );
    assert_eq!(g.players[1].life, life - 3, "unblocked wither is normal life loss");
    assert_eq!(g.players[1].poison_counters, 0, "wither gives players no poison");
}

// ── CR 702.180b — Harmonize: tap a creature to reduce the cost ────────────────

/// Tapping a creature reduces a Harmonize cost by that creature's power in
/// generic mana (CR 702.180b). Wild Ride's harmonize {4}{R} shrinks to {R}
/// when a 4-power creature is tapped.
#[test]
fn cr_702_180b_harmonize_tap_reduces_cost() {
    let mut g = two_player_game();
    let ride = g.add_card_to_graveyard(0, catalog::wild_ride());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tapper = g.add_card_to_battlefield(0, catalog::hill_giant()); // power 3
    g.clear_sickness(tapper);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // {4}{R} harmonize − 3 (tapped power) = {1}{R}. Pay exactly that.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastHarmonize {
        card_id: ride,
        tap_creature: Some(tapper),
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Wild Ride via harmonize, reduced by the tapped 4-power creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tapper).unwrap().tapped, "the creature was tapped for the reduction");
    assert_eq!(g.computed_permanent(target).unwrap().power, 5, "resolved: +3/+0 on the 2/2");
}

// ── CR 514.2 — "until end of turn" grants end at cleanup ──────────────────────

/// A granted-harmonize (Songcrafter Mage) is an until-end-of-turn effect; it
/// expires during the cleanup step (CR 514.2), even on a graveyard card.
#[test]
fn cr_514_2_granted_harmonize_expires_at_cleanup() {
    let mut g = two_player_game();
    let div = g.add_card_to_graveyard(0, catalog::divination());
    g.move_card_to_battlefield_for_test(0, catalog::songcrafter_mage());
    drain_stack(&mut g);
    assert!(
        g.players[0].graveyard.iter().find(|c| c.id == div).unwrap().effective_harmonize().is_some(),
        "harmonize granted this turn"
    );
    // Advance into the opponent's turn — this turn's cleanup runs in between.
    while g.active_player_idx == 0 {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(
        g.players[0].graveyard.iter().find(|c| c.id == div).unwrap().effective_harmonize().is_none(),
        "the grant ended at cleanup (CR 514.2)",
    );
}

// ── CR 614.5 — one-sided damage doubler spares the controller's own side ──────

/// Twinflame Tyrant / Gisela's doubler applies only to damage dealt to
/// opponents; damage to your own permanents is not doubled (CR 614.5 scoping).
#[test]
fn cr_614_5_one_sided_doubler_spares_own_side() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::twinflame_tyrant());
    let own = g.add_card_to_battlefield(0, catalog::hill_giant()); // your 3/3
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // Bolt your own creature — the opponent-only doubler must NOT apply.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(own)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt own creature");
    drain_stack(&mut g);
    // 3 damage (not 6) to a 3/3 → it dies exactly, having taken 3.
    assert!(g.battlefield_find(own).is_none(), "took lethal 3 (undoubled) and died");
    // Opponent unaffected.
    assert_eq!(g.players[1].life, 20, "no damage leaked to the opponent");
}

// ── CR 702.121 — Melee ───────────────────────────────────────────────────────

/// A creature with melee gets +1/+1 until end of turn for each opponent it
/// attacked this combat (one, in a duel).
#[test]
fn cr_702_121_melee_pumps_per_opponent_attacked() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let m = g.add_card_to_battlefield(0, catalog::menagerie_liberator()); // 3/2, trample+melee
    g.clear_sickness(m);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: m,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(m).unwrap().power, 4, "melee: +1/+1 for the one opponent");
}

// ── CR 614.5 — additive damage-replacement (typed) ───────────────────────────

/// Valley Flamecaller's static adds 1 to the damage a typed creature you control
/// deals — an additive CR 614.5 replacement applied before doublers.
#[test]
fn cr_614_5_typed_creatures_deal_extra_damage() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.players[1].life = 20;
    let fc = g.add_card_to_battlefield(0, catalog::valley_flamecaller()); // 3/3 Lizard
    g.clear_sickness(fc);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: fc,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    cr_advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 16, "3 power + 1 = 4 combat damage");
}

// ── CR 401 — library / graveyard recursion ───────────────────────────────────

/// `Effect::ShuffleGraveyardCardsIntoLibrary` moves up to N chosen graveyard
/// cards into the owner's library (Cathartic Parting's recursion rider).
#[test]
fn cr_401_shuffle_graveyard_cards_into_library() {
    use crabomination::effect::{Effect, PlayerRef, Value};
    use crabomination::card::SelectionRequirement as R;
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let lib_before = g.players[0].library.len();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(src, 0, None);
    g.resolve_effect(
        &Effect::ShuffleGraveyardCardsIntoLibrary {
            who: PlayerRef::You,
            filter: R::Any,
            max: Value::Const(4),
        },
        &ctx,
    )
    .unwrap();
    assert_eq!(g.players[0].graveyard.len(), 1, "four of five reshuffled out of the graveyard");
    assert_eq!(g.players[0].library.len(), lib_before + 4, "into the library");
}

// ── CR 702.172 — Spree (choose one or more modes; costs fold) ─────────────────

/// CR 702.172a — casting a Spree spell pays its base cost plus the costs of each
/// chosen mode, and applies each chosen mode. Explosive Derailment ({R}; +{2}
/// deal 4 to a creature, +{2} destroy an artifact) with both modes costs {4}{R}
/// and resolves both.
#[test]
fn cr_702_172_spree_folds_costs_and_applies_all_modes() {
    let mut g = two_player_game();
    g.step = crabomination::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let creature = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let artifact = g.add_card_to_battlefield(1, catalog::the_everflowing_well());
    let id = g.add_card_to_hand(0, catalog::explosive_derailment());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4); // {R} + {2} + {2}
    g.perform_action(GameAction::CastSpellSpree {
        card_id: id,
        spree_modes: vec![0, 1],
        target: Some(crabomination::game::types::Target::Permanent(creature)),
        additional_targets: vec![crabomination::game::types::Target::Permanent(artifact)],
        x_value: None,
    })
    .expect("both modes affordable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(creature).is_none(), "mode 0 dealt 4 to the 4/4");
    assert!(g.battlefield_find(artifact).is_none(), "mode 1 destroyed the artifact");
}

/// CR 702.172c — a Spree spell requires at least one chosen mode; casting with
/// no modes is illegal.
#[test]
fn cr_702_172_spree_requires_at_least_one_mode() {
    let mut g = two_player_game();
    g.step = crabomination::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::explosive_derailment());
    g.players[0].mana_pool.add(Color::Red, 1);
    let res = g.perform_action(GameAction::CastSpellSpree {
        card_id: id,
        spree_modes: vec![],
        target: None,
        additional_targets: vec![],
        x_value: None,
    });
    assert!(res.is_err(), "zero modes is not a legal Spree cast");
}

// ── CR 603.10a — self-death "this or another" trigger ────────────────────────

/// CR 603.10a — a `YourControl` "whenever this creature or another creature you
/// control dies" trigger looks back at the dying creature and fires for its own
/// death. Zulaport Cutthroat drains when it itself dies.
#[test]
fn cr_603_10a_self_death_trigger_fires() {
    let mut g = two_player_game();
    let zula = g.add_card_to_battlefield(0, catalog::zulaport_cutthroat()); // 1/1
    let opp = g.players[1].life;
    g.battlefield_find_mut(zula).unwrap().damage = 1; // lethal
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "Zulaport drained on its own death");
}

// ── CR 724 — "End the turn" ends "until end of turn" effects ─────────────────
/// CR 724.2 — Time Stop ends the turn, so a resolved "until end of turn" pump
/// wears off immediately as the turn is cleaned up.
#[test]
fn cr_724_end_the_turn_ends_until_eot_effects() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: growth, target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Giant Growth");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "pumped +3/+3");
    let stop = g.add_card_to_hand(0, catalog::time_stop());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: stop, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Time Stop");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "the EOT pump ended with the turn");
}

// ── CR 115.7 — "you may choose new targets for target spell" ─────────────────
/// Bolt Bend repoints a single-target spell at a new target of its caster's
/// choice (CR 115.7 — the redirect mutates the original spell in place).
#[test]
fn cr_115_7_bolt_bend_repoints_a_spell() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    // Opponent bolts your bear.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent bolts the bear");
    // You Bolt Bend it, repointing the bolt at the opponent.
    let bend = g.add_card_to_hand(0, catalog::bolt_bend());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Target(crabomination::game::types::Target::Player(1)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: bend, target: Some(crabomination::game::types::Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bolt Bend on the bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "the bear was spared");
    assert_eq!(g.players[1].life, 17, "the bolt was redirected to its caster");
}

// ── CR 702.72 — Deathtouch: any nonzero combat damage is lethal ─────────────
/// A 1/1 deathtoucher kills a 5/6 blocker with a single point of combat damage
/// (CR 702.2e — SBA marks it destroyed).
#[test]
fn cr_702_2_deathtouch_one_damage_is_lethal() {
    use crabomination::game::types::Attack;
    let mut g = two_player_game();
    let rat = g.add_card_to_battlefield(0, catalog::typhoid_rats()); // 1/1 deathtouch
    g.clear_sickness(rat);
    let djinn = g.add_card_to_battlefield(1, catalog::mahamoti_djinn()); // 5/6
    g.attacking = vec![Attack { attacker: rat, target: AttackTarget::Player(1) }];
    g.set_block_map([(djinn, rat)]);
    g.step = TurnStep::CombatDamage;
    g.active_player_idx = 0;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(djinn).is_none(), "5/6 destroyed by 1 deathtouch damage");
}

/// CR 614.2 — Gratuitous Violence doubles combat damage dealt by a creature its
/// controller controls (source-restricted, so the opponent's swing is normal).
#[test]
fn cr_614_2_gratuitous_violence_doubles_controlled_creature_combat_damage() {
    g_614_2_helper(0, 1, 16); // your 2/2 attacks: 2 → 4, opponent 20 → 16
    g_614_2_helper(1, 0, 18); // opponent's 2/2 attacks: undoubled 2, you 20 → 18
}

fn g_614_2_helper(attacker_seat: usize, defender_seat: usize, expected_life: i32) {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gratuitous_violence());
    let bear = g.add_card_to_battlefield(attacker_seat, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(defender_seat) }];
    g.step = TurnStep::CombatDamage;
    g.active_player_idx = attacker_seat;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.players[defender_seat].life, expected_life);
}

/// CR 601.2d + 701.10 — Biogenic Upgrade distributes three +1/+1 counters among
/// two targets (2/1, the even split) then doubles the counters on each of those
/// targets via `Selector::AllTargets` (→ 4 and 2).
#[test]
fn cr_601_2d_distribute_counters_then_double_on_all_targets() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::biogenic_upgrade());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    }).expect("cast Biogenic Upgrade on two targets");
    drain_stack(&mut g);
    let n = |id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    // 3 split 2/1, then each doubled.
    assert_eq!((n(a), n(b)), (4, 2), "distributed 2/1 then doubled to 4/2");
}

/// CR 702.17 — Flash lets a permanent be cast any time you could cast an instant
/// (here, during the opponent's turn); a creature without Flash can't.
#[test]
fn cr_702_17_flash_permanent_castable_at_instant_speed() {
    let mut g = two_player_game();
    g.active_player_idx = 1; // opponent's turn
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let flashy = g.add_card_to_hand(0, catalog::wildborn_preserver()); // {1}{G} Flash
    let vanilla = g.add_card_to_hand(0, catalog::grizzly_bears()); // no Flash
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: vanilla, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "a vanilla creature can't be cast at instant speed");
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: flashy, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_ok(), "Flash lets it resolve on the opponent's turn");
}

/// CR 700.14 — "expend N" counts the total mana a player spends to cast spells
/// in a turn and triggers when the running total first reaches N. A cheaper
/// spell (2 mana) doesn't reach 4; a 6-mana spell crosses it, and the payoff
/// fires only once even though the total keeps climbing.
#[test]
fn cr_700_14_expend_four_triggers_once_when_crossed() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::teapot_slinger()); // expend 4 → 2 to each opp
    // Cross expend 4 with a 6-mana spell — the payoff fires once.
    let first = g.add_card_to_hand(0, catalog::galewind_moose()); // {4}{G}{G}, 6 mana
    let second = g.add_card_to_hand(0, catalog::galewind_moose());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: first, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast 6-mana spell (crosses expend 4)");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "expend 4 pings once");
    // A second spell the same turn is already past the threshold — no re-ping.
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: second, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a second 6-mana spell");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "expend 4 doesn't re-fire later the same turn");
}

/// CR 702.108 (Raid) — a Raid trigger checks whether its controller attacked
/// this turn. Alesha's end-step reanimation fires only after she (or another
/// creature) has attacked; with no attack, the graveyard creature stays put.
#[test]
fn cr_702_108_raid_end_step_requires_an_attack() {
    // No attack this turn → the Raid trigger's condition fails, no reanimation.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::alesha_who_laughs_at_fate());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cr_advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == dead),
        "no attack this turn → Raid doesn't reanimate");
}

/// CR 602.5 — an activated ability marked "Activate only once" (once per game,
/// not once per turn) stays spent across turn boundaries. Mild-Mannered
/// Librarian can't re-transform even on a later turn.
#[test]
fn cr_602_5_activate_once_persists_across_turns() {
    let mut g = two_player_game();
    let lib = g.add_card_to_battlefield(0, catalog::mild_mannered_librarian());
    g.add_card_to_library(0, catalog::forest());
    g.clear_sickness(lib);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lib, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("first activation");
    drain_stack(&mut g);
    // Simulate a fresh turn: clear the per-turn used state the way cleanup would.
    g.battlefield_find_mut(lib).unwrap().once_per_turn_used.clear();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: lib, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "once-per-game ability stays spent across turns");
}

/// CR 702.12 — Indestructible granted by a counter (Myojin of Night's Reach's
/// divinity counter via `SelfHasKeywordWhileCountersAtLeast`) stops a "destroy"
/// effect; removing the last counter makes it destructible again.
#[test]
fn cr_702_12_counter_granted_indestructible_survives_destroy() {
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let myojin = g.add_card_to_battlefield(0, catalog::myojin_of_nights_reach());
    g.battlefield_find_mut(myojin).unwrap().add_counters(CounterType::Divinity, 1);
    let destroy = crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) };
    let ctx = EffectContext {
        targets: vec![crabomination::game::types::Target::Permanent(myojin)],
        ..EffectContext::for_trigger(myojin, 0, None, 0)
    };
    g.resolve_effect(&destroy, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(myojin).is_some(), "indestructible while it has a divinity counter");
    g.battlefield_find_mut(myojin).unwrap().remove_counters(CounterType::Divinity, 1);
    g.resolve_effect(&destroy, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(myojin).is_none(), "destructible once the counter is gone");
}

/// CR 510.1c / 122.1 — a "deals combat damage to a player" trigger reads the
/// damage dealt (`Value::TriggerEventAmount`); Drake Hatcher (1/3) banks one
/// incubation counter per point of combat damage.
#[test]
fn cr_510_1c_combat_damage_banks_incubation_counters() {
    let mut g = two_player_game();
    let hatcher = g.add_card_to_battlefield(0, catalog::drake_hatcher()); // 1/3
    g.clear_sickness(hatcher);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hatcher,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(hatcher).unwrap().counter_count(CounterType::Incubation),
        1,
        "1 combat damage → 1 incubation counter"
    );
}

/// CR 118.4 / 122.1 — a cost that removes N counters is unpayable if the source
/// has fewer than N. Drake Hatcher's "remove three incubation counters" ability
/// can't be activated with only two.
#[test]
fn cr_118_remove_counter_cost_requires_enough_counters() {
    let mut g = two_player_game();
    let hatcher = g.add_card_to_battlefield(0, catalog::drake_hatcher());
    g.clear_sickness(hatcher);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.battlefield_find_mut(hatcher).unwrap().add_counters(CounterType::Incubation, 2);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: hatcher,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None, mode: None,
        })
        .is_err(),
        "can't remove three counters with only two"
    );
    g.battlefield_find_mut(hatcher).unwrap().add_counters(CounterType::Incubation, 1);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: hatcher,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None, mode: None,
        })
        .is_ok(),
        "activatable once a third counter lands"
    );
}

/// CR 500.4/106.4 — "you don't lose this mana as steps and phases end": mana
/// kept this turn survives a step/phase empty and clears at cleanup.
#[test]
fn cr_500_4_kept_mana_survives_step_empty() {
    let mut g = two_player_game();
    let vent = g.add_card_to_battlefield(0, catalog::savage_ventmaw());
    let effect = catalog::savage_ventmaw().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &crabomination::game::effects::EffectContext::for_trigger(vent, 0, None, 0)).unwrap();
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.total(), 6, "kept mana survives the empty");
    g.players[0].kept_mana_this_turn.empty();
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.total(), 0, "cleared at cleanup");
}

/// CR 701.60 — a suspected creature has menace and can't block.
#[test]
fn cr_701_60_suspected_creature_has_menace_and_cant_block() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.resolve_effect(
        &crabomination::effect::Effect::Suspect { what: crabomination::effect::Selector::Target(0) },
        &crabomination::game::effects::EffectContext { targets: vec![crabomination::game::types::Target::Permanent(bear)], ..crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0) },
    ).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Menace), "suspected → menace");
    assert!(cp.keywords.contains(&Keyword::CantBlock), "suspected → can't block");
}

/// CR 614 — Doorkeeper Thrull suppresses ETB triggers of entering *artifacts*
/// (not just creatures): the entering artifact's trigger multiplier is 0.
#[test]
fn cr_614_artifact_etb_triggers_suppressed_by_doorkeeper_thrull() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::doorkeeper_thrull());
    let sal = g.add_card_to_battlefield(0, catalog::sandstorm_salvager());
    assert_eq!(
        crabomination::game::actions::etb_trigger_multiplier(&g, 0, Some(sal)),
        0,
        "an entering artifact fires no ETB triggers under the suppressor",
    );
}

// ── CR 115 / 601.2c — Effect::OptionalTargets declinable slot ────────────────

/// CR 601.2c — Primal Might's fight target is "up to one"; cast against your own
/// creature with no opposing creature present, the optional fight slot is simply
/// skipped and the spell still pumps the chosen creature.
#[test]
fn cr_601_2c_optional_targets_fight_slot_is_skippable() {
    use crabomination::game::{drain_stack, GameAction, TurnStep};
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::primal_might());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2); // X = 2
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("castable targeting only your creature — the fight target is optional");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 4, "+2/+2 applied without a fight");
}

/// CR 709.5 — Rampaging Soulrager's static reads the count of unlocked doors
/// among Rooms you control via `Predicate::UnlockedDoorsControlledAtLeast`.
#[test]
fn cr_709_5_unlocked_doors_gate_self_pump() {
    let mut g = two_player_game();
    let sr = g.add_card_to_battlefield(0, catalog::rampaging_soulrager());
    let room = g.add_card_to_battlefield(0, catalog::roaring_furnace_steaming_sauna());
    assert_eq!(g.computed_permanent(sr).unwrap().power, 1, "no doors → 1/4");
    g.battlefield_find_mut(room).unwrap().unlock_room_door(false);
    assert_eq!(g.computed_permanent(sr).unwrap().power, 1, "one door is not enough");
    g.battlefield_find_mut(room).unwrap().unlock_room_door(true);
    assert_eq!(g.computed_permanent(sr).unwrap().power, 4, "two unlocked doors → +3/+0");
}

// ── CR 702.16e — protection from a creature *type* prevents combat damage ─────

/// CR 702.16e — Resilient Roadrunner has protection from Coyotes, so a Coyote's
/// combat damage to it is prevented (the protection check keys on creature type,
/// not just color/creatures).
#[test]
fn cr_702_16e_protection_from_creature_type_prevents_damage() {
    let mut g = two_player_game();
    let roadrunner = g.add_card_to_battlefield(0, catalog::resilient_roadrunner());
    let coyote = g.add_card_to_battlefield(1, catalog::driftgloom_coyote());
    assert!(
        g.damage_prevented_by_protection(coyote, roadrunner),
        "a Coyote can't deal combat damage to a creature with protection from Coyotes",
    );
    assert!(
        !g.damage_prevented_by_protection(roadrunner, coyote),
        "the Roadrunner still damages the Coyote normally",
    );
}

// ── CR 122.5 — moving counters relocates every kind, keyword counters too ──
#[test]
fn cr_122_5_move_all_counters_relocates_keyword_counters() {
    use crabomination::card::Keyword;
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx_a = EffectContext::for_ability(a, 0, None);
    g.resolve_effect(
        &Effect::AddKeywordCounter { what: Selector::This, keyword: Keyword::Flying, amount: Value::ONE },
        &ctx_a,
    )
    .unwrap();
    assert!(g.computed_permanent(a).unwrap().keywords.contains(&Keyword::Flying));
    let ctx_move = EffectContext { targets: vec![Target::Permanent(b)], ..EffectContext::for_ability(a, 0, None) };
    g.resolve_effect(&Effect::MoveAllCounters { from: Selector::This, to: Selector::Target(0) }, &ctx_move).unwrap();
    assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::Flying), "keyword counter relocated");
    assert!(!g.computed_permanent(a).unwrap().keywords.contains(&Keyword::Flying), "source lost it");
}

// ── CR 702.166 — manifest dread N times, then counters on those creatures ──
#[test]
fn cr_702_166_manifest_dread_repeat_puts_counters() {
    use crabomination::card::CounterType;
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let ctx = EffectContext::for_spell(0, None, 0, 2); // X = 2
    g.resolve_effect(&catalog::valgavoths_onslaught().effect, &ctx).unwrap();
    let facedown: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).collect();
    assert_eq!(facedown.len(), 2);
    assert!(facedown.iter().all(|c| c.counter_count(CounterType::PlusOnePlusOne) == 2));
}

// ── CR 608.2b — a resolution-time "if greatest power" gate on a destroy ──
#[test]
fn cr_608_2b_conditional_destroy_gated_by_greatest_power() {
    use crabomination::effect::Effect;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let destroy = match &catalog::getaway_glamer().effect {
        Effect::Spree { modes } => modes[1].effect.clone(),
        _ => panic!("not spree"),
    };
    // Targeting the smaller creature: another creature has greater power, no destroy.
    let ctx_small = EffectContext { targets: vec![Target::Permanent(small)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&destroy, &ctx_small).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_some(), "no creature has less power to gate on — spared");
    // Targeting the biggest creature: destroyed.
    let ctx_big = EffectContext { targets: vec![Target::Permanent(big)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&destroy, &ctx_big).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "greatest-power creature destroyed");
}

// ── CR 702.165 — a permanent Gift is given as it enters ──────────────────────

/// CR 702.165 — casting a creature with its Gift promised gives the gift as the
/// permanent enters, firing "whenever you give a gift" payoffs (Jolly Gerbils).
#[test]
fn cr_702_165_permanent_gift_fires_give_a_gift_payoff() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::jolly_gerbils());
    g.add_card_to_library(0, catalog::forest()); // Jolly Gerbils' draw
    g.add_card_to_library(1, catalog::forest()); // Scrapshooter's gift draw
    g.add_card_to_battlefield(1, catalog::sol_ring()); // a legal ETB destroy target
    let scrap = g.add_card_to_hand(0, catalog::scrapshooter());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastGift {
        card_id: scrap, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Scrapshooter with gift");
    drain_stack(&mut g);
    // hand: -1 for the creature spell, +1 for Jolly Gerbils = net hand.
    assert_eq!(g.players[0].hand.len(), hand, "the permanent gift fired Jolly Gerbils");
}

// ── CR 613.7c — conditional self-pump stacks on counters in layer 7 ──────────

/// CR 613.7c — a +1/+1 counter and a conditional +2/+2 (Aven Heartstabber) both
/// apply in layer 7 and stack additively on the printed 1/1.
#[test]
fn cr_613_7c_counter_and_conditional_pump_stack() {
    let mut g = two_player_game();
    let aven = g.add_card_to_battlefield(0, catalog::aven_heartstabber());
    g.battlefield_find_mut(aven).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    // Five distinct mana values in the graveyard switch on the +2/+2.
    for c in [catalog::forest(), catalog::grizzly_bears(), catalog::sol_ring(),
              catalog::horses_of_the_bruinen(), catalog::eagle_of_deliverance()] {
        g.add_card_to_graveyard(0, c);
    }
    assert_eq!(g.computed_permanent(aven).unwrap().power, 4, "1 base + 1 counter + 2 pump");
    assert_eq!(g.computed_permanent(aven).unwrap().toughness, 4);
}

// ── CR 202.3b — {X} in a card's cost is 0 outside the stack ──────────────────

/// CR 202.3b — a card with {X} in its graveyard is mana value = its non-X part,
/// so Aven Heartstabber's distinct-mana-value count treats Fireball ({X}{R}) as
/// MV 1.
#[test]
fn cr_202_3b_x_cost_is_zero_in_graveyard() {
    let mut g = two_player_game();
    let aven = g.add_card_to_battlefield(0, catalog::aven_heartstabber());
    // MV set {1 (Fireball), 0 (Forest), 2 (Bears), 1 (Sol Ring), 6 (Eagle)} →
    // distinct {0,1,2,6} = 4 values, one short of the five-value threshold.
    for c in [catalog::fireball(), catalog::forest(), catalog::grizzly_bears(),
              catalog::sol_ring(), catalog::eagle_of_deliverance()] {
        g.add_card_to_graveyard(0, c);
    }
    assert_eq!(g.computed_permanent(aven).unwrap().power, 1, "only four distinct MVs (X counts as 0)");
    // A fifth distinct value (MV 5) flips it on.
    g.add_card_to_graveyard(0, catalog::horses_of_the_bruinen()); // {3}{U}{U} = 5
    assert_eq!(g.computed_permanent(aven).unwrap().power, 3, "five distinct MVs → +2/+2");
}

// ── CR 701.5g / 709.3 / 608.2h (this run's DIS gap wave) ─────────────────────

/// CR 701.5g — a countered spell is put into its owner's graveyard. Swift
/// Silence counters every other spell on the stack; the countered card lands
/// in the graveyard (not exile).
#[test]
fn cr_701_5g_countered_spell_goes_to_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear onto the stack");
    let silence = g.add_card_to_hand(0, catalog::swift_silence());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: silence, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Swift Silence");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear),
        "the countered spell rests in its owner's graveyard");
}

/// CR 709.3 — a split card's two halves are cast independently, each paying its
/// own cost and resolving only its own effect. Punishment (right) destroys by
/// mana value X without touching Crime's reanimation.
#[test]
fn cr_709_3_split_halves_cast_independently() {
    let mut g = two_player_game();
    let two_drop = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let three_drop = g.add_card_to_battlefield(1, catalog::craw_wurm()); // MV 6
    let cp = g.add_card_to_hand(0, catalog::crime_punishment());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSplitRight {
        card_id: cp, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("Punishment castable for X=2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(two_drop).is_none(), "MV-2 destroyed by Punishment");
    assert!(g.battlefield_find(three_drop).is_some(), "MV-6 untouched");
}

/// CR 608.2h — a permanent sacrificed as part of resolution is measured by its
/// last-known information. Hit reads the sacrificed permanent's mana value
/// after it has already left the battlefield.
#[test]
fn cr_608_2h_sacrificed_mana_value_read_via_lki() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::craw_wurm()); // MV 6, the only sac fodder
    let hr = g.add_card_to_hand(0, catalog::hit_run());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: hr, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hit");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 6, "damage equals the departed permanent's LKI mana value");
}

// ── CR 606.3 — borrowed loyalty abilities share the once-per-turn budget ──────
/// Nicol Bolas, Dragon-God gains every other planeswalker's loyalty abilities,
/// but CR 606.3's "only once each turn" is per *permanent*: activating one
/// (even a borrowed) ability spends Bolas's activation for the turn.
#[test]
fn cr_606_3_borrowed_loyalty_shares_once_per_turn() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bolas = g.add_card_to_battlefield(0, catalog::nicol_bolas_dragon_god());
    g.battlefield_find_mut(bolas).unwrap().counters.insert(CounterType::Loyalty, 8);
    g.add_card_to_battlefield(0, catalog::chandra_fire_artisan());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Activate a borrowed ability (Chandra's +1 at index 3).
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: bolas, ability_index: 3, target: None, x_value: None }).expect("first activation");
    drain_stack(&mut g);
    // A second activation the same turn — even Bolas's own −3 — is rejected.
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let err = g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: bolas, ability_index: 1, target: Some(Target::Permanent(foe)), x_value: None }).unwrap_err();
    assert!(matches!(err, GameError::LoyaltyAbilityAlreadyUsed(id) if id == bolas), "one loyalty activation per turn per permanent");
}

// ── CR 601.2h — an unpayable life cost blocks the cast ────────────────────────
/// A spell cast off Bolas's Citadel pays life equal to its mana value; if the
/// player can't pay that life, the cast is illegal (CR 601.2h).
#[test]
fn cr_601_2h_bolass_citadel_unpayable_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bolass_citadel());
    // A high-MV spell on top; life too low to pay it.
    let wurm = g.add_card_to_library(0, catalog::craw_wurm()); // MV 6
    g.players[0].life = 3;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let err = g.perform_action(GameAction::CastSpell { card_id: wurm, target: None, additional_targets: vec![], mode: None, x_value: None }).unwrap_err();
    assert!(matches!(err, GameError::InsufficientLife), "can't pay 6 life at 3 life");
    assert!(g.players[0].library.first().is_some_and(|c| c.id == wurm), "spell stays on top of library");
}

// ── CR 706 — Finale of Promise copies each free-cast spell twice at X ≥ 10 ─────
/// With X ≥ 10, Finale of Promise copies the instant and sorcery it casts twice
/// each; the copies are new objects on the stack (CR 707.10).
#[test]
fn cr_707_10_finale_of_promise_copies_at_ten() {
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let opp_life = g.players[1].life;
    let ctx = EffectContext {
        targets: vec![Target::Permanent(bolt)],
        ..EffectContext::for_spell(0, None, 0, 10) // X = 10
    };
    g.resolve_effect(&catalog::finale_of_promise().effect, &ctx).unwrap();
    drain_stack(&mut g);
    // Original Bolt + two copies = 9 damage to the opponent.
    assert_eq!(g.players[1].life, opp_life - 9, "Bolt plus two copies dealt 9");
}

/// Swords to Plowshares — "Exile target creature. Its controller gains
/// life equal to its power." Regression for the 2026-07 audit fix that
/// added the lifegain rider (it was exile-only before).
#[test]
fn swords_to_plowshares_controller_gains_power_in_life() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::swords_to_plowshares());
    g.players[0].mana_pool.add(Color::White, 1);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("StP castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == angel), "Angel exiled");
    assert_eq!(g.players[1].life, p1 + 4,
        "its controller gains life equal to its power (4)");
}

/// Teferi's Protection (audit fix): the life-lock drops both loss and
/// gain this turn, and the spell exiles itself on resolution.
#[test]
fn teferis_protection_locks_life_and_self_exiles() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::teferis_protection());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("TP castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == id), "exiles itself");
    // Life is locked: a Lightning Bolt-style life loss is dropped.
    let before = g.players[0].life;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before, "life total can't change");
}

// ── CR 614 — player-damage and life-gain replacements (Odyssey rares) ────────

/// CR 614.1b — Delaying Shield replaces damage dealt to its controller with
/// delay counters on itself; the life total never moves.
#[test]
fn cr_614_1b_player_damage_replaced_with_counters_on_source() {
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    let shield = g.add_card_to_battlefield(0, catalog::delaying_shield());
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 4, None, &mut events);
    assert_eq!(g.players[0].life, 20);
    assert_eq!(g.battlefield_find(shield).unwrap().counter_count(CounterType::Delay), 4);
}

/// CR 614.1b — Nefarious Lich pays damage out of the graveyard, and loses the
/// game when the graveyard can't cover it.
#[test]
fn cr_614_1b_player_damage_replaced_with_graveyard_exile() {
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nefarious_lich());
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 2, None, &mut events);
    assert_eq!(g.players[0].life, 20);
    assert!(g.players[0].graveyard.is_empty());
    assert!(!g.players[0].eliminated);
    g.deal_damage_to_from(EntityRef::Player(0), 1, None, &mut events);
    assert!(g.players[0].eliminated, "no cards left to pay with");
}

/// CR 614.1b — Nefarious Lich's second replacement turns life gain into draws,
/// so the gain event never happens.
#[test]
fn cr_614_1b_life_gain_replaced_with_draws() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nefarious_lich());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand = g.players[0].hand.len();
    g.adjust_life(0, 3);
    assert_eq!(g.players[0].life, 20, "no life was gained");
    assert_eq!(g.players[0].hand.len(), hand + 3);
    assert_eq!(g.players[0].life_gained_this_turn, 0, "no life-gain event fired");
}

// ── CR 702.16 — protection from its own colors ──────────────────────────────

/// CR 702.16e — "protection from its colors" is a real blocking restriction:
/// under Earnest Fellowship a green attacker can't be blocked by a green
/// creature.
#[test]
fn cr_702_16e_protection_from_own_colors_blocks_same_color_blockers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::earnest_fellowship());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    g.battlefield_find_mut(attacker).unwrap().summoning_sick = false;
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).is_err(),
        "a green blocker can't block a creature with protection from green"
    );
    // A colourless blocker is unaffected.
    let artifact = g.add_card_to_battlefield(1, catalog::ornithopter());
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(artifact, attacker)])).is_ok());
}

// ── CR 701.7 — destroying a permanent ───────────────────────────────────────

/// CR 701.7a — "destroy" by a spell or ability an opponent controls is a
/// distinct event from a combat/SBA death, and only the cross-team case fires
/// the watcher (Karmic Justice).
#[test]
fn cr_701_7a_opponent_destroy_fires_the_retaliation_watcher() {
    use crabomination::card::SelectionRequirement;
    use crabomination::effect::{Effect, Selector};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::karmic_justice());
    let mine = g.add_card_to_battlefield(0, catalog::catalyst_stone());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(mine, 1, None);
    let events = g
        .resolve_effect(
            &Effect::Destroy {
                what: Selector::EachPermanent(SelectionRequirement::Artifact),
            },
            &ctx,
        )
        .expect("destroy");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentDestroyedByEffect { .. })),
        "the destroy funnel emitted the CR 701.7 event"
    );
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none());
}

/// CR 701.7a — destroying your *own* permanent emits no cross-team event, so
/// the watcher stays quiet.
#[test]
fn cr_701_7a_self_destroy_does_not_fire_the_watcher() {
    use crabomination::card::SelectionRequirement;
    use crabomination::effect::{Effect, Selector};
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::catalyst_stone());
    let ctx = crabomination::game::effects::EffectContext::for_ability(mine, 0, None);
    let events = g
        .resolve_effect(
            &Effect::Destroy {
                what: Selector::EachPermanent(SelectionRequirement::Artifact),
            },
            &ctx,
        )
        .expect("destroy");
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentDestroyedByEffect { .. }))
    );
}

// ── CR 121.2a — replacing a draw ────────────────────────────────────────────

/// CR 121.2a — "you may skip that draw instead" is a real optional draw
/// replacement: declining draws normally.
#[test]
fn cr_121_2a_controller_may_skip_a_draw() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::obstinate_familiar());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut events = Vec::new();
    assert!(!g.draw_one(0, &mut events), "the skip was taken");
    assert!(g.draw_one(0, &mut events), "declining draws normally");
}

