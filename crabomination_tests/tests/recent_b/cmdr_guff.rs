//! Commander: the Planeswalker Party precon (CMM, Commodore Guff,
//! `decks::cmdr_guff`). The primitives it needed are tested in
//! `core_rules/commander_cards.rs`.

use crabomination::card::{CardId, CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, mode: Option<usize>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    if let Some(c) = g.battlefield_find_mut(id) {
        c.loyalty_uses_this_turn = 0;
    }
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
}

fn loyalty_of(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map(|c| c.counter_count(CounterType::Loyalty)).unwrap_or(0)
}

fn connect(g: &mut GameState, attacker: CardId, at: usize) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(at) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    g.step = TurnStep::PostCombatMain;
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

/// CR 606 — Guff's +1 Wizard, −3 per-walker draw and burn, and the end-step
/// loyalty counter on another walker.
#[test]
fn commodore_guff_parties() {
    let mut g = pod(3);
    library(&mut g, 0, 4);
    let guff = g.add_card_to_battlefield(0, catalog::commodore_guff());
    let teyo = g.add_card_to_battlefield(0, catalog::teyo_geometric_tactician());
    loyalty(&mut g, guff, 0, None, None).expect("+1");
    assert_eq!(named(&g, 0, "Wizard").len(), 1);
    let before = loyalty_of(&g, teyo);
    step(&mut g, TurnStep::End);
    assert_eq!(loyalty_of(&g, teyo), before + 1);
    g.step = TurnStep::PreCombatMain;
    let (hand, life) = (g.players[0].hand.len(), g.players[1].life);
    loyalty(&mut g, guff, 1, None, None).expect("−3");
    assert_eq!(g.players[0].hand.len(), hand + 2);
    assert_eq!(g.players[1].life, life - 2);
}

/// CR 606 — Chandra, Awakened Inferno's −3 spares Elementals; −X exiles
/// what it kills; +2 gives each opponent a burning emblem.
#[test]
fn chandra_awakened_inferno_burns() {
    let mut g = pod(2);
    let chandra = g.add_card_to_battlefield(0, catalog::chandra_awakened_inferno());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let leori = g.add_card_to_battlefield(1, catalog::leori_sparktouched_hunter());
    loyalty(&mut g, chandra, 1, None, None).expect("−3");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(leori).is_some(), "an Elemental");
    g.battlefield_find_mut(chandra).unwrap().counters.insert(CounterType::Loyalty, 6);
    loyalty(&mut g, chandra, 2, Some(Target::Permanent(leori)), Some(3)).expect("−3 as X");
    assert!(g.exile.iter().any(|c| c.id == leori), "exiled instead of dying");
    loyalty(&mut g, chandra, 0, None, None).expect("+2");
    let life = g.players[1].life;
    g.active_player_idx = 1;
    step(&mut g, TurnStep::Upkeep);
    assert_eq!(g.players[1].life, life - 1);
}

/// CR 606 — Chandra, Legacy of Fire adds {R} per walker and burns per walker
/// at your end step.
#[test]
fn chandra_legacy_of_fire_scales_with_walkers() {
    let mut g = pod(2);
    let chandra = g.add_card_to_battlefield(0, catalog::chandra_legacy_of_fire());
    g.add_card_to_battlefield(0, catalog::teyo_geometric_tactician());
    loyalty(&mut g, chandra, 0, None, None).expect("+1");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2);
    let life = g.players[1].life;
    step(&mut g, TurnStep::End);
    assert_eq!(g.players[1].life, life - 2);
}

/// CR 701.23 — Deploy the Gatewatch puts two planeswalkers from the top
/// seven onto the battlefield.
#[test]
fn deploy_the_gatewatch_deploys_two() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    g.add_card_to_library(0, catalog::teyo_geometric_tactician());
    g.add_card_to_library(0, catalog::jace_mirror_mage());
    g.add_card_to_library(0, catalog::island());
    let deploy = g.add_card_to_hand(0, catalog::deploy_the_gatewatch());
    cast(&mut g, deploy, None, None).expect("cast");
    let walkers = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_planeswalker()).count();
    assert_eq!(walkers, 2);
}

/// CR 122.1 — Gatewatch Beacon enters with three loyalty counters and hands
/// one to an entering planeswalker.
#[test]
fn gatewatch_beacon_feeds_walkers() {
    let mut g = pod(2);
    let beacon = g.add_card_to_hand(0, catalog::gatewatch_beacon());
    cast(&mut g, beacon, None, None).expect("cast");
    assert_eq!(loyalty_of(&g, beacon), 3);
    let teyo = g.add_card_to_hand(0, catalog::teyo_geometric_tactician());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    cast(&mut g, teyo, None, None).expect("cast");
    assert_eq!(loyalty_of(&g, teyo), 4);
    assert_eq!(loyalty_of(&g, beacon), 2);
}

/// CR 701.24 — Guff Rewrites History swaps an opponent's permanent for the
/// next spell off their library.
#[test]
fn guff_rewrites_history_reshuffles_a_threat() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let guff = g.add_card_to_hand(0, catalog::guff_rewrites_history());
    cast(&mut g, guff, Some(Target::Permanent(ring)), None).expect("cast");
    assert!(g.battlefield_find(ring).is_some_and(|c| c.controller == 1), "its only library card, recast");
    assert!(g.players[1].library.is_empty());
}

/// CR 702.33 — Jace, Mirror Mage kicked makes a one-loyalty copy; its 0
/// pays a drawn card's mana value in loyalty.
#[test]
fn jace_mirror_mage_mirrors() {
    let mut g = pod(2);
    let jace = g.add_card_to_hand(0, catalog::jace_mirror_mage());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: jace,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked");
    drain_stack(&mut g);
    let jaces = named(&g, 0, "Jace, Mirror Mage");
    assert_eq!(jaces.len(), 2, "the copy isn't legendary");
    let token = *jaces.iter().find(|&&id| id != jace).unwrap();
    assert_eq!(loyalty_of(&g, token), 1);
    g.add_card_to_library(0, catalog::grizzly_bears());
    loyalty(&mut g, jace, 1, None, None).expect("0");
    assert_eq!(loyalty_of(&g, jace), 2, "four less the Bears' two");
}

/// CR 707.10 — Jaya's Phoenix's hit copies the next loyalty ability; a
/// planeswalker spell brings it back from the graveyard.
#[test]
fn jayas_phoenix_copies_and_rises() {
    let mut g = pod(2);
    let phoenix = g.add_card_to_battlefield(0, catalog::jayas_phoenix());
    let guff = g.add_card_to_battlefield(0, catalog::commodore_guff());
    connect(&mut g, phoenix, 1);
    loyalty(&mut g, guff, 0, None, None).expect("+1");
    assert_eq!(named(&g, 0, "Wizard").len(), 2, "the Wizard and its copy");
    let _ = g.remove_to_graveyard_with_triggers(phoenix);
    let teyo = g.add_card_to_hand(0, catalog::teyo_geometric_tactician());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    cast(&mut g, teyo, None, None).expect("a planeswalker spell");
    assert!(g.battlefield_find(phoenix).is_some());
}

/// CR 707.10 — Leori's hit copies the chosen type's abilities this turn.
#[test]
fn leori_copies_a_planeswalker_type() {
    let mut g = pod(2);
    let leori = g.add_card_to_battlefield(0, catalog::leori_sparktouched_hunter());
    let teyo = g.add_card_to_battlefield(0, catalog::teyo_geometric_tactician());
    library(&mut g, 0, 3);
    library(&mut g, 1, 3);
    connect(&mut g, leori, 1);
    let hand = g.players[0].hand.len();
    loyalty(&mut g, teyo, 0, Some(Target::Player(1)), None).expect("+1");
    assert_eq!(g.players[0].hand.len(), hand + 2, "Teyo's draw and its copy");
}

/// CR 606 — Narset of the Ancient Way's +1 and −2.
#[test]
fn narset_of_the_ancient_way_gains_and_loots() {
    let mut g = pod(2);
    let narset = g.add_card_to_battlefield(0, catalog::narset_of_the_ancient_way());
    let life = g.players[0].life;
    loyalty(&mut g, narset, 0, None, None).expect("+1");
    assert_eq!(g.players[0].life, life + 2);
    assert_eq!(g.players[0].mana_pool.restricted_total(), 1, "spend only on noncreature spells");
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::solemn_simulacrum());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    loyalty(&mut g, narset, 1, Some(Target::Permanent(bear)), None).expect("−2");
    assert!(g.battlefield_find(bear).is_none(), "the discarded Solemn's four damage");
}

/// CR 601.2 — Narset, Enlightened Master's attack frees noncreature spells.
#[test]
fn narset_enlightened_master_frees_spells() {
    let mut g = pod(2);
    let narset = g.add_card_to_battlefield(0, catalog::narset_enlightened_master());
    let ring = g.add_card_to_library(0, catalog::sol_ring());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    library(&mut g, 0, 2);
    connect(&mut g, narset, 1);
    assert!(g.exile.iter().find(|c| c.id == ring).is_some_and(|c| c.may_play_until.is_some()));
    assert!(g.exile.iter().find(|c| c.id == bear).is_some_and(|c| c.may_play_until.is_none()), "a creature");
}

/// CR 606.3 — Oath of Teferi flickers a permanent until the end step and
/// lets walkers activate twice.
#[test]
fn oath_of_teferi_flickers_and_doubles() {
    let mut g = pod(2);
    let teyo = g.add_card_to_battlefield(0, catalog::teyo_geometric_tactician());
    let oath = g.add_card_to_hand(0, catalog::oath_of_teferi());
    cast(&mut g, oath, Some(Target::Permanent(teyo)), None).expect("cast");
    assert!(g.battlefield_find(teyo).is_none());
    step(&mut g, TurnStep::End);
    let teyo = named(&g, 0, "Teyo, Geometric Tactician")[0];
    library(&mut g, 0, 3);
    library(&mut g, 1, 3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    for _ in 0..2 {
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: teyo,
            ability_index: 0,
            target: Some(Target::Player(1)),
            x_value: None,
        })
        .expect("twice");
        drain_stack(&mut g);
    }
}

/// CR 508.1g — Onakke taxes attacks on planeswalkers; from the graveyard it
/// returns one.
#[test]
fn onakke_oathkeeper_returns_a_walker() {
    let mut g = pod(2);
    let onakke = g.add_card_to_graveyard(0, catalog::onakke_oathkeeper());
    let teyo = g.add_card_to_graveyard(0, catalog::teyo_geometric_tactician());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: onakke,
        ability_index: 0,
        target: Some(Target::Permanent(teyo)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("from the graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(teyo).is_some());
    assert!(g.exile.iter().any(|c| c.id == onakke));
}

/// CR 707.10 — Repeated Reverberation copies the next instant twice.
#[test]
fn repeated_reverberation_triples_a_bolt() {
    let mut g = pod(2);
    let rr = g.add_card_to_hand(0, catalog::repeated_reverberation());
    cast(&mut g, rr, None, None).expect("cast");
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)), None).expect("bolt");
    assert_eq!(g.players[1].life, life - 9);
}

/// CR 613.1d — Sparkshaper Visionary turns your walkers into 3/3 flyers.
#[test]
fn sparkshaper_visionary_makes_birds() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::sparkshaper_visionary());
    let teyo = g.add_card_to_battlefield(0, catalog::teyo_geometric_tactician());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    step(&mut g, TurnStep::BeginCombat);
    let c = g.computed_permanent(teyo).expect("on the battlefield");
    assert_eq!((c.power, c.toughness), (3, 3));
    assert!(c.card_types().contains(&CardType::Creature));
    assert!(g.permanent_has_keyword(teyo, &Keyword::Flying));
}

/// CR 606 — Teyo makes a Wall and draws for two.
#[test]
fn teyo_walls_and_draws() {
    let mut g = pod(2);
    let teyo = g.add_card_to_hand(0, catalog::teyo_geometric_tactician());
    cast(&mut g, teyo, None, None).expect("cast");
    assert_eq!(named(&g, 0, "Wall").len(), 1);
    library(&mut g, 0, 2);
    library(&mut g, 1, 2);
    let (a, b) = (g.players[0].hand.len(), g.players[1].hand.len());
    loyalty(&mut g, teyo, 0, Some(Target::Player(1)), None).expect("+1");
    assert_eq!((g.players[0].hand.len(), g.players[1].hand.len()), (a + 1, b + 1));
}

/// CR 702.26 — Vronos's +1 phases the others out at the end step; −2
/// bounces; −7 makes a 9/9.
#[test]
fn vronos_masks_and_bounces() {
    let mut g = pod(2);
    let vronos = g.add_card_to_battlefield(0, catalog::vronos_masked_inquisitor());
    let teyo = g.add_card_to_battlefield(0, catalog::teyo_geometric_tactician());
    loyalty(&mut g, vronos, 0, None, None).expect("+1");
    step(&mut g, TurnStep::End);
    assert!(g.phased_out.iter().any(|c| c.id == teyo));
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    loyalty(&mut g, vronos, 1, Some(Target::Permanent(bear)), None).expect("−2");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    g.battlefield_find_mut(vronos).unwrap().counters.insert(CounterType::Loyalty, 7);
    loyalty(&mut g, vronos, 2, Some(Target::Permanent(ring)), None).expect("−7");
    let c = g.computed_permanent(ring).unwrap();
    assert_eq!((c.power, c.toughness), (9, 9));
}

