//! Commander: the 2026-10-19 (`016Xh16H`) closers (`decks::cmdr_016x_a`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

/// CR 601.2b-c — Cruel Alliance's teamwork is announced before its target is
/// chosen: unteamed it can't reach a mana-value-4 creature; teamed (a 2-power
/// creature tapped) it exiles one and gains 3 life.
#[test]
fn cruel_alliance_teamwork_widens_the_target() {
    let mut g = pod(3);
    let giant = ready(&mut g, 2, catalog::hill_giant());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ca = g.add_card_to_hand(0, catalog::cruel_alliance());
    flood(&mut g, 0);
    assert!(cast(&mut g, ca, Some(Target::Permanent(giant))).is_err(), "MV 4 > 3 unteamed");
    g.perform_action(GameAction::CastSpellKicked {
        card_id: ca,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("teamwork cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == giant));
    assert_eq!(g.players[0].life, 23);
    assert!(g.battlefield_find(bear).unwrap().tapped, "the bear paid teamwork");
}

/// Cruel Alliance unteamed exiles a small creature and gains no life.
#[test]
fn cruel_alliance_unteamed_exiles_a_small_creature() {
    let mut g = pod(3);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let ca = g.add_card_to_hand(0, catalog::cruel_alliance());
    flood(&mut g, 0);
    cast(&mut g, ca, Some(Target::Permanent(bear))).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bear));
    assert_eq!(g.players[0].life, 20);
}

/// Glyph Keeper counters the first spell to target it each turn; the second
/// resolves.
#[test]
fn glyph_keeper_counters_the_first_targeting_spell() {
    let mut g = pod(2);
    let gk = ready(&mut g, 1, catalog::glyph_keeper());
    flood(&mut g, 0);
    let b1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let b2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, b1, Some(Target::Permanent(gk))).expect("bolt 1");
    assert_eq!(g.battlefield_find(gk).map(|c| c.damage), Some(0), "countered");
    cast(&mut g, b2, Some(Target::Permanent(gk))).expect("bolt 2");
    assert!(g.battlefield_find(gk).is_none(), "the second bolt kills it");
}

/// CR 702.128a — Vizier of the Anointed entombs a creature card with embalm,
/// and activating that embalm draws a card (the trait reads the activation,
/// not the token it makes).
#[test]
fn vizier_of_the_anointed_entombs_and_draws_on_embalm() {
    let mut g = pod(2);
    let gk = g.add_card_to_library(0, catalog::glyph_keeper());
    let vz = g.add_card_to_hand(0, catalog::vizier_of_the_anointed());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, vz, None).expect("vizier");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == gk), "entombed");
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: gk,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
        mode: None,
    })
    .expect("embalm");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "embalm draws");
    let tok = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Glyph Keeper").expect("token");
    assert_eq!(tok.definition.printed_colors(), vec![Color::White], "embalm makes it white");
}

/// CR 700.2 — Splinter & Leo's default line: the counters on your own team,
/// the Mutant to a different player (each mode targets a different player).
#[test]
fn splinter_and_leo_counters_your_team() {
    let mut g = pod(3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sl = g.add_card_to_hand(0, catalog::splinter_leo_father_son());
    flood(&mut g, 0);
    cast(&mut g, sl, None).expect("cast");
    let bear_counters = g.battlefield_find(bear).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne);
    assert_eq!(bear_counters, 1);
    assert_eq!(g.battlefield_find(sl).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 0, "other creatures");
    let mutants: Vec<usize> = g.battlefield.iter().filter(|c| c.definition.name == "Mutant").map(|c| c.controller).collect();
    assert!(mutants.iter().all(|&s| s != 0), "the token mode went to another player: {mutants:?}");
}

/// CR 615 — Djeru shaves 1 off damage to a planeswalker you control.
#[test]
fn djeru_shields_your_planeswalkers() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    ready(&mut g, 1, catalog::djeru_with_eyes_open());
    let lili = ready(&mut g, 1, catalog::liliana_death_mage());
    let start = g.battlefield_find(lili).unwrap().counter_count(CounterType::Loyalty);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Permanent(lili))).expect("bolt");
    assert_eq!(g.battlefield_find(lili).unwrap().counter_count(CounterType::Loyalty), start - 2);
}

/// Elspeth's Talent grants "[+1]: three Soldiers"; activating it (a loyalty
/// ability of the enchanted planeswalker) pumps the team +2/+2 and vigilance.
#[test]
fn elspeths_talent_grants_soldiers_and_pumps_on_loyalty() {
    let mut g = pod(2);
    let lili = ready(&mut g, 0, catalog::liliana_death_mage());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let et = g.add_card_to_hand(0, catalog::elspeths_talent());
    flood(&mut g, 0);
    cast(&mut g, et, Some(Target::Permanent(lili))).expect("cast");
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: lili, ability_index: 3, target: None, x_value: None })
        .expect("granted +1");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 3);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (4, 4));
    assert!(b.has_keyword(&crabomination::card::Keyword::Vigilance));
}

// ── Old Hob, Kang, Ultimo, Mila (`decks::cmdr_016x_b`) ──────────────────────

fn pass_to(g: &mut GameState, step: TurnStep) {
    for _ in 0..200 {
        if g.step == step {
            return;
        }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    panic!("never reached {step:?}");
}

/// Old Hob's combat Mutant has haste and is destroyed at the next end step.
#[test]
fn old_hob_makes_a_hasty_mutant_that_dies_at_end_step() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::old_hob_alleycat_blues());
    pass_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    let m = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Mutant").expect("mutant").id;
    assert!(g.battlefield_find(m).unwrap().has_keyword(&crabomination::card::Keyword::Haste));
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(m).is_none(), "destroyed at the end step");
}

/// CR 603.7 — Rewrite History's fourth plan counter sacrifices it, and the
/// reflexive trigger returns an instant from your graveyard.
#[test]
fn rewrite_history_fourth_plan_counter_returns_spells() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let rh = ready(&mut g, 0, catalog::rewrite_history());
    g.battlefield_find_mut(rh).unwrap().counters.insert(CounterType::Plan, 3);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: bear,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rh).is_none(), "sacrificed at four");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "bolt returned");
}

/// Gix's Caress takes a nonland card and leaves a tapped Powerstone.
#[test]
fn gixs_caress_discards_a_nonland_and_makes_a_powerstone() {
    let mut g = pod(2);
    g.players[1].hand.clear();
    let land = g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let gc = g.add_card_to_hand(0, catalog::gixs_caress());
    flood(&mut g, 0);
    cast(&mut g, gc, Some(Target::Player(1))).expect("cast");
    let hand: Vec<CardId> = g.players[1].hand.iter().map(|c| c.id).collect();
    assert_eq!(hand, vec![land]);
    let ps = g.battlefield.iter().find(|c| c.definition.name == "Powerstone").expect("powerstone");
    assert!(ps.tapped);
}

/// Okiba Salvage's rider needs both an artifact and an enchantment.
#[test]
fn okiba_salvage_counters_with_artifact_and_enchantment() {
    use crabomination::card::CounterType;
    for (both, want) in [(true, 2), (false, 0)] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::sol_ring());
        if both {
            ready(&mut g, 0, catalog::rewrite_history());
        }
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let os = g.add_card_to_hand(0, catalog::okiba_salvage());
        flood(&mut g, 0);
        cast(&mut g, os, Some(Target::Permanent(bear))).expect("cast");
        let c = g.battlefield_find(bear).expect("reanimated");
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), want, "both={both}");
    }
}

/// Norritt forces the active player's creature to attack; one that never
/// did is destroyed at the next end step.
#[test]
fn norritt_destroys_a_creature_that_did_not_attack() {
    let mut g = pod(2);
    let norritt = ready(&mut g, 0, catalog::norritt());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: norritt,
        ability_index: 1,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&crabomination::card::Keyword::MustAttack));
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 1;
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "never attacked: destroyed");
}

/// CR 104.3d / 114.4 — Gideon of the Trials' emblem keeps you in the game at
/// 0 life only while you control a Gideon planeswalker.
#[test]
fn gideon_of_the_trials_emblem_needs_a_gideon() {
    let mut g = pod(3);
    let gid = ready(&mut g, 0, catalog::gideon_of_the_trials());
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: gid, ability_index: 2, target: None, x_value: None })
        .expect("emblem");
    drain_stack(&mut g);
    g.players[0].life = 0;
    g.check_state_based_actions();
    assert!(!g.players[0].eliminated, "Gideon out: still in");
    g.battlefield_find_mut(gid).unwrap().counters.insert(crabomination::card::CounterType::Loyalty, 0);
    g.check_state_based_actions();
    assert!(g.battlefield_find(gid).is_none());
    g.check_state_based_actions();
    assert!(g.players[0].eliminated, "no Gideon: the loss lands");
}

// ── Quickbeam, Elesh Norn ───────────────────────────────────────────────────

/// Progenitor Exarch at X = 2 incubates 3 twice; transforming one with
/// Norn's Inquisitor out gives the Phyrexian a +1/+1 counter (4 in all).
#[test]
fn progenitor_exarch_incubates_x_times_and_inquisitor_grows_the_phyrexian() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    ready(&mut g, 0, catalog::norns_inquisitor());
    let pe = g.add_card_to_hand(0, catalog::progenitor_exarch());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: pe, target: None, additional_targets: vec![], mode: None, x_value: Some(2) })
        .expect("cast");
    drain_stack(&mut g);
    let incs: Vec<CardId> = g.battlefield.iter().filter(|c| c.definition.name == "Incubator").map(|c| c.id).collect();
    assert_eq!(incs.len(), 2);
    g.clear_sickness(pe);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pe,
        ability_index: 0,
        target: Some(Target::Permanent(incs[0])),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("transform");
    drain_stack(&mut g);
    let p = g.battlefield_find(incs[0]).unwrap();
    assert_eq!(p.counter_count(CounterType::PlusOnePlusOne), 4, "3 + the Inquisitor's 1");
}

/// Tiller of Flesh incubates on a spell targeting a permanent, not a player.
#[test]
fn tiller_of_flesh_incubates_on_permanent_targets() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::tiller_of_flesh());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    flood(&mut g, 0);
    let b1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let b2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let n = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Incubator").count();
    cast(&mut g, b1, Some(Target::Player(1))).expect("face");
    assert_eq!(n(&g), 0);
    cast(&mut g, b2, Some(Target::Permanent(bear))).expect("bear");
    assert_eq!(n(&g), 1);
}

/// Lumberknot grows on any creature's death.
#[test]
fn lumberknot_grows_when_a_creature_dies() {
    let mut g = pod(2);
    let lk = ready(&mut g, 0, catalog::lumberknot());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("bolt");
    assert_eq!(g.battlefield_find(lk).unwrap().power(), 2);
}

/// CR 614.1a — Deification names its controller's planeswalker type as it
/// enters; with a creature out, lethal damage to that walker leaves one
/// loyalty counter, and without one it dies. The walker has hexproof.
#[test]
fn deification_leaves_the_chosen_walker_at_one_loyalty() {
    use crabomination::card::{CounterType, Keyword};
    for with_creature in [true, false] {
        let mut g = pod(2);
        let lili = ready(&mut g, 0, catalog::liliana_death_mage());
        if with_creature {
            ready(&mut g, 0, catalog::grizzly_bears());
        }
        let d = g.add_card_to_hand(0, catalog::deification());
        flood(&mut g, 0);
        cast(&mut g, d, None).expect("deification");
        assert!(g.permanent_has_keyword(lili, &Keyword::Hexproof), "hexproof from the static");
        g.battlefield_find_mut(lili).unwrap().counters.insert(CounterType::Loyalty, 3);
        let dmg = crabomination::effect::Effect::DealDamage {
            to: crabomination::effect::Selector::Target(0),
            amount: crabomination::card::Value::Const(5),
        };
        let mut ctx = EffectContext::for_spell(1, None, 0, 0);
        ctx.targets = vec![Target::Permanent(lili)];
        let evs = g.resolve_effect(&dmg, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        g.check_state_based_actions();
        match with_creature {
            true => assert_eq!(g.battlefield_find(lili).unwrap().counter_count(CounterType::Loyalty), 1),
            false => assert!(g.battlefield_find(lili).is_none(), "no creature: it dies"),
        }
    }
}

// ── Tomik, Orzhov Lawmage / Tobias Andrion ──────────────────────────────────

/// Way of the Healer empowers Jace 5, and that Jace has the granted [−2]:
/// a Cadet, then surveil 1.
#[test]
fn way_of_the_healer_empowers_and_grants_cadets() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let woh = g.add_card_to_hand(0, catalog::way_of_the_healer());
    flood(&mut g, 0);
    cast(&mut g, woh, None).expect("cast");
    let jace = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.is_planeswalker())
        .expect("a Jace token")
        .id;
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 5);
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: jace, ability_index: 2, target: None, x_value: None })
        .expect("granted −2");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Cadet"));
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 3);
}

/// CR 509.1d — Awesome Presence: a blocker on the enchanted attacker costs
/// {3}; with no mana the block is refused, with it the block stands.
#[test]
fn awesome_presence_taxes_blockers_of_its_host() {
    for pay in [false, true] {
        let mut g = pod(2);
        let atk = ready(&mut g, 0, catalog::grizzly_bears());
        let blk = ready(&mut g, 1, catalog::grizzly_bears());
        let ap = g.add_card_to_hand(0, catalog::awesome_presence());
        flood(&mut g, 0);
        cast(&mut g, ap, Some(Target::Permanent(atk))).expect("aura");
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
            attacker: atk,
            target: crabomination::game::types::AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        if pay {
            g.players[1].mana_pool.add_colorless(3);
        }
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let res = g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)]));
        assert_eq!(res.is_ok(), pay, "pay={pay}");
    }
}
