//! Commander: the Mishra's Burnished Banner precon (BRC, Mishra, Eminent One,
//! `decks::cmdr_mishra`). The primitives it needed are tested in
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

fn cast_x(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate_x(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    activate_x(g, id, index, target, None)
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

/// CR 707.9a — Mishra's token copies the noncreature artifact and is also a
/// hasty 4/4 artifact creature; CR 603.7 — it's sacrificed at the next end step.
#[test]
fn mishra_makes_a_hasty_warform_that_dies_at_end_step() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::mishra_eminent_one());
    g.add_card_to_battlefield(0, catalog::mind_stone());
    step(&mut g, TurnStep::BeginCombat);
    let stones = named(&g, 0, "Mind Stone");
    assert_eq!(stones.len(), 2, "the Stone and its Warform");
    let warform = *stones.iter().find(|&&id| g.battlefield_find(id).unwrap().is_token).expect("a token");
    let card = g.battlefield_find(warform).unwrap();
    assert!(card.definition.card_types.contains(&CardType::Creature));
    assert_eq!(pt(&g, warform), (4, 4));
    assert!(g.permanent_has_keyword(warform, &Keyword::Haste));
    step(&mut g, TurnStep::End);
    assert!(g.battlefield_find(warform).is_none(), "sacrificed at the end step");
}

/// CR 707.10 — Ashnod copies an ability whose cost sacrificed a permanent:
/// Fain's sacrifice-an-artifact ability makes two Inklings.
#[test]
fn ashnod_copies_a_sacrifice_ability() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::ashnod_the_uncaring());
    let fain = ready(&mut g, 0, catalog::fain_the_broker());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    activate(&mut g, fain, 2, None).expect("sacrifice the Ornithopter");
    assert_eq!(named(&g, 0, "Inkling").len(), 2, "the ability and its copy");
}

/// CR 702.4a — creatures attacking an opponent under Blast-Furnace Hellkite
/// have double strike.
#[test]
fn blast_furnace_hellkite_gives_attackers_double_strike() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::blast_furnace_hellkite());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let life = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(bear, &Keyword::DoubleStrike));
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, life - 4);
}

/// CR 602.2 — Fain sacrifices a creature for two +1/+1 counters, then turns a
/// counter into a Treasure; {3}{B} untaps it between the two.
#[test]
fn fain_trades_creatures_for_counters_and_counters_for_treasure() {
    let mut g = pod(2);
    let fain = ready(&mut g, 0, catalog::fain_the_broker());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, fain, 0, Some(Target::Permanent(fain))).expect("sacrifice the Bears");
    assert_eq!(g.battlefield_find(fain).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    activate(&mut g, fain, 3, None).expect("untap");
    activate(&mut g, fain, 1, None).expect("a counter for a Treasure");
    assert_eq!(g.battlefield_find(fain).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(named(&g, 0, "Treasure").len(), 1);
}

/// CR 603.6c — a nontoken artifact dying makes Farid a Scrap; the Scrap then
/// pays for the counter-and-menace mode.
#[test]
fn farid_scraps_dead_artifacts() {
    let mut g = pod(2);
    let farid = ready(&mut g, 0, catalog::farid_enterprising_salvager());
    let bomb = g.add_card_to_battlefield(0, catalog::ornithopter());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(0)]));
    activate(&mut g, farid, 0, None).expect("sacrifice the Ornithopter");
    assert!(g.battlefield_find(bomb).is_none());
    assert_eq!(named(&g, 0, "Scrap").len(), 1, "a nontoken artifact died");
    assert_eq!(g.battlefield_find(farid).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.permanent_has_keyword(farid, &Keyword::Menace));
}

/// CR 107.3 — Geth's X is the returned card's mana value; its owner mills X.
#[test]
fn geth_reanimates_an_opponents_card_and_mills_them() {
    let mut g = pod(2);
    let geth = ready(&mut g, 0, catalog::geth_lord_of_the_vault());
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    activate_x(&mut g, geth, 0, Some(Target::Permanent(bear)), Some(2)).expect("X=2");
    let card = g.battlefield_find(bear).expect("under our control");
    assert_eq!(card.controller, 0);
    assert!(card.tapped);
    assert_eq!(g.players[1].graveyard.len(), 2, "milled two");
}

/// CR 613.4c — Glint Raker gets +X/+0 for the biggest artifact.
#[test]
fn glint_raker_grows_with_the_biggest_artifact() {
    let mut g = pod(2);
    let raker = g.add_card_to_battlefield(0, catalog::glint_raker());
    assert_eq!(pt(&g, raker).0, 1);
    g.add_card_to_battlefield(0, catalog::mind_stone());
    assert_eq!(pt(&g, raker).0, 3);
}

/// CR 704.5f — Herald of Anguish's -2/-2 kills a Bear; an end step discard.
#[test]
fn herald_of_anguish_shrinks_and_strips_hands() {
    let mut g = pod(2);
    let herald = g.add_card_to_battlefield(0, catalog::herald_of_anguish());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, herald, 0, Some(Target::Permanent(bear))).expect("sacrifice the Ornithopter");
    assert!(g.battlefield_find(bear).is_none());
    g.add_card_to_hand(1, catalog::island());
    step(&mut g, TurnStep::End);
    assert!(g.players[1].hand.is_empty());
}

/// CR 707.10 — Lithoform Engine copies an instant you control.
#[test]
fn lithoform_engine_copies_a_bolt() {
    let mut g = pod(2);
    let engine = g.add_card_to_battlefield(0, catalog::lithoform_engine());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let life = g.players[1].life;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    activate(&mut g, engine, 1, Some(Target::Permanent(bolt))).expect("copy it");
    assert_eq!(g.players[1].life, life - 6);
}

/// CR 707.9b — Machine God's Effigy copies a creature but is a noncreature
/// artifact that taps for {U}.
#[test]
fn machine_gods_effigy_copies_a_creature_as_a_mana_rock() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let effigy = g.add_card_to_hand(0, catalog::machine_gods_effigy());
    cast_x(&mut g, effigy, None, None).expect("cast");
    let card = g.battlefield_find(effigy).expect("on the battlefield");
    assert_eq!(card.definition.name, "Grizzly Bears");
    assert!(!card.definition.card_types.contains(&CardType::Creature));
    assert!(card.definition.card_types.contains(&CardType::Artifact));
    assert!(!card.definition.activated_abilities.is_empty(), "it taps for {{U}}");
}

/// CR 603.2h — Oni-Cult Anvil's sacrifice drains each opponent and, being an
/// artifact leaving on our turn, makes one Construct.
#[test]
fn oni_cult_anvil_drains_and_builds() {
    let mut g = pod(3);
    let anvil = g.add_card_to_battlefield(0, catalog::oni_cult_anvil());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    let (me, a, b) = (g.players[0].life, g.players[1].life, g.players[2].life);
    activate(&mut g, anvil, 0, None).expect("sacrifice one");
    assert_eq!((g.players[0].life, g.players[1].life, g.players[2].life), (me + 1, a - 1, b - 1));
    assert_eq!(named(&g, 0, "Construct").len(), 1);
    g.battlefield_find_mut(anvil).unwrap().tapped = false;
    activate(&mut g, anvil, 0, None).expect("sacrifice the other");
    assert_eq!(named(&g, 0, "Construct").len(), 1, "only once each turn");
}

/// CR 702.34 — Scavenged Brawler's graveyard ability puts four +1/+1 counters
/// and four keyword counters on a creature.
#[test]
fn scavenged_brawler_upgrades_a_creature_from_the_graveyard() {
    let mut g = pod(2);
    let brawler = g.add_card_to_graveyard(0, catalog::scavenged_brawler());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, brawler, 0, Some(Target::Permanent(bear))).expect("from the graveyard");
    assert_eq!(pt(&g, bear), (6, 6));
    for k in [Keyword::Flying, Keyword::Vigilance, Keyword::Trample, Keyword::Lifelink] {
        assert!(g.permanent_has_keyword(bear, &k), "{k:?}");
    }
    assert!(g.exile.iter().any(|c| c.id == brawler));
}

/// CR 701.20 — Smelting Vat puts noncreature artifacts no bigger than the
/// sacrificed one from the top eight onto the battlefield.
#[test]
fn smelting_vat_digs_for_artifacts() {
    let mut g = pod(2);
    let vat = g.add_card_to_battlefield(0, catalog::smelting_vat());
    g.add_card_to_battlefield(0, catalog::mind_stone());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_library(0, catalog::sol_ring());
    g.add_card_to_library(0, catalog::lithoform_engine());
    g.add_card_to_library(0, catalog::mind_stone());
    activate(&mut g, vat, 0, None).expect("sacrifice the Stone");
    assert_eq!(named(&g, 0, "Sol Ring").len(), 1);
    assert_eq!(named(&g, 0, "Mind Stone").len(), 1, "the new one");
    assert!(named(&g, 0, "Lithoform Engine").is_empty(), "mana value 4 is too big");
}

/// CR 107.3 — Terisiare's Devastation makes X tapped Powerstones first, then
/// shrinks everything by the artifact count.
#[test]
fn terisiares_devastation_counts_its_own_powerstones() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    let dev = g.add_card_to_hand(0, catalog::terisiares_devastation());
    cast_x(&mut g, dev, None, Some(2)).expect("X=2");
    assert_eq!(g.players[0].life, life - 2);
    let stones = named(&g, 0, "Powerstone");
    assert_eq!(stones.len(), 2);
    assert!(stones.iter().all(|&id| g.battlefield_find(id).unwrap().tapped));
    assert!(g.battlefield_find(bear).is_none(), "-2/-2");
}

/// CR 502.3 — Traxos doesn't untap normally but a historic spell untaps it.
#[test]
fn traxos_untaps_on_historic_spells() {
    let mut g = pod(2);
    let traxos = g.add_card_to_hand(0, catalog::traxos_scourge_of_kroog());
    cast_x(&mut g, traxos, None, None).expect("cast");
    assert!(g.battlefield_find(traxos).unwrap().tapped, "enters tapped");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_x(&mut g, bolt, Some(Target::Player(1)), None).expect("bolt");
    assert!(g.battlefield_find(traxos).unwrap().tapped, "a bolt isn't historic");
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast_x(&mut g, ring, None, None).expect("ring");
    assert!(!g.battlefield_find(traxos).unwrap().tapped);
}

/// CR 707.12 — Wondrous Crucible mills two, exiles the nonland one and casts
/// a copy of it: a Bear token.
#[test]
fn wondrous_crucible_casts_a_milled_copy() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::wondrous_crucible());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    step(&mut g, TurnStep::End);
    assert!(g.exile.iter().any(|c| c.id == bear), "the card stays exiled");
    let tokens = named(&g, 0, "Grizzly Bears");
    assert_eq!(tokens.len(), 1, "the copy became a token");
}

/// CR 702.21a — Wondrous Crucible gives our permanents ward {2}.
#[test]
fn wondrous_crucible_grants_ward() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::wondrous_crucible());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.computed_permanent(bear).expect("on the battlefield");
    assert!(c.keywords().iter().any(|k| matches!(k, Keyword::Ward(_))));
}

/// CR 613.1d — Workshop Elders animates a noncreature artifact into a 4/4
/// with four +1/+1 counters. (Its flying grant reads printed types, so the
/// animated artifact doesn't fly yet — ENGINE_BACKLOG, CR 613.8.)
#[test]
fn workshop_elders_animates_an_artifact() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::workshop_elders());
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(pt(&g, stone), (4, 4));
    let thopter = g.add_card_to_battlefield(0, catalog::ornithopter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.permanent_has_keyword(thopter, &Keyword::Flying));
    assert!(!g.permanent_has_keyword(bear, &Keyword::Flying), "not an artifact");
}
