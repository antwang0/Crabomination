//! Commander: the Desert Bloom precon (OTC, Yuma, `decks::cmdr_yuma`).

use crabomination::card::{CardId, Keyword, LandType};
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
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_x(g, seat, id, target, None)
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn sac(g: &mut GameState, seat: usize, id: CardId) {
    let mut evs = Vec::new();
    g.sacrifice_one(id, seat, &mut evs);
    drain_stack(g);
}

/// Hazezon — a Desert of yours entering makes two Sand Warriors; Deserts are
/// playable from your graveyard.
#[test]
fn hazezon_makes_warriors_and_replays_deserts() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::hazezon_shaper_of_sand());
    let d = g.add_card_to_graveyard(0, catalog::painted_bluffs());
    g.perform_action(GameAction::PlayLandFromGraveyard(d)).expect("play the Desert from the graveyard");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Painted Bluffs").len(), 1);
    assert_eq!(named(&g, 0, "Sand Warrior").len(), 2);
    let h = named(&g, 0, "Hazezon, Shaper of Sand")[0];
    assert!(g.computed_permanent(h).unwrap().keywords().contains(&Keyword::Landwalk(LandType::Desert)));
}

/// Yuma — a Desert card reaching your graveyard from anywhere (here, milled)
/// makes a 4/2 Plant Warrior; Dunes of the Dead dying makes a Zombie.
#[test]
fn yuma_grows_plants_from_deserts() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::yuma_proud_protector());
    g.add_card_to_library(0, catalog::painted_bluffs());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let pt_ = g.add_card_to_battlefield(0, catalog::perpetual_timepiece());
    activate(&mut g, 0, pt_, 0).expect("mill two");
    assert_eq!(named(&g, 0, "Plant Warrior").len(), 1, "Yuma");
    let d = g.add_card_to_battlefield(0, catalog::dunes_of_the_dead());
    sac(&mut g, 0, d);
    assert_eq!(named(&g, 0, "Zombie").len(), 1, "Dunes of the Dead");
}

/// Yuma's cost falls with land cards in your graveyard.
#[test]
fn yuma_costs_less_per_graveyard_land() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    let y = g.add_card_to_hand(0, catalog::yuma_proud_protector());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: y, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{R}{G}{W} with five lands in the graveyard");
}

/// Sand Scout fetches a Desert when behind on lands, and lands reaching your
/// graveyard make a Sand Warrior once a turn.
#[test]
fn sand_scout_fetches_and_mints() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::painted_bluffs());
    g.add_card_to_battlefield(1, catalog::forest());
    let s = g.add_card_to_hand(0, catalog::sand_scout());
    cast(&mut g, 0, s, None).expect("cast");
    let pb = named(&g, 0, "Painted Bluffs");
    assert_eq!(pb.len(), 1);
    assert!(g.battlefield_find(pb[0]).unwrap().tapped);
    // Two land cards milled at once, then a third: one Warrior (once a turn).
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let pt_ = g.add_card_to_battlefield(0, catalog::perpetual_timepiece());
    activate(&mut g, 0, pt_, 0).expect("mill two");
    assert_eq!(named(&g, 0, "Sand Warrior").len(), 1, "once each turn");
}

/// Kirri — other Plants get +2/+0; your postcombat main returns a land card.
#[test]
fn kirri_pumps_plants_and_recurs_lands() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::kirri_talented_sprout());
    let dc = g.add_card_to_battlefield(0, catalog::dune_chanter());
    assert_eq!(pt(&g, dc), (4, 3));
    g.add_card_to_graveyard(0, catalog::forest());
    let h = g.players[0].hand.len();
    g.step = TurnStep::PostCombatMain;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h + 1);
}

/// Dune Chanter — your lands are Deserts; {T}: mill two, a life per land.
#[test]
fn dune_chanter_makes_deserts_and_mills() {
    let mut g = pod(2);
    let dc = g.add_card_to_battlefield(0, catalog::dune_chanter());
    let f = g.add_card_to_battlefield(0, catalog::forest());
    assert!(g.computed_permanent(f).unwrap().subtypes().land_types.contains(&LandType::Desert));
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.clear_sickness(dc);
    let l = g.players[0].life;
    activate(&mut g, 0, dc, 0).expect("mill");
    assert_eq!(g.players[0].life, l + 2);
}

/// Cataclysmic Prospecting deals X to each creature.
#[test]
fn cataclysmic_prospecting_sweeps() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let cp = g.add_card_to_hand(0, catalog::cataclysmic_prospecting());
    cast_x(&mut g, 0, cp, None, Some(2)).expect("cast");
    assert!(named(&g, 1, "Grizzly Bears").is_empty());
    assert!(g.battlefield_find(giant).is_some());
}

/// Descend upon the Sinful exiles all creatures; delirium adds an Angel.
#[test]
fn descend_upon_the_sinful_exiles_and_delirium_angel() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for f in [catalog::forest, catalog::grizzly_bears, catalog::lightning_bolt, catalog::sol_ring] {
        g.add_card_to_graveyard(0, f());
    }
    let d = g.add_card_to_hand(0, catalog::descend_upon_the_sinful());
    cast(&mut g, 0, d, None).expect("cast");
    assert!(named(&g, 1, "Grizzly Bears").is_empty());
    assert_eq!(named(&g, 0, "Angel").len(), 1);
}

/// Vengeful Regrowth returns land cards tapped with a Plant Warrior each.
#[test]
fn vengeful_regrowth_returns_lands_and_plants() {
    let mut g = pod(2);
    let a = g.add_card_to_graveyard(0, catalog::forest());
    let b = g.add_card_to_graveyard(0, catalog::forest());
    let vr = g.add_card_to_hand(0, catalog::vengeful_regrowth());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: vr,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Forest").len(), 2);
    assert_eq!(named(&g, 0, "Plant Warrior").len(), 2);
}

/// World Shaper's death returns every land card in your graveyard tapped.
#[test]
fn world_shaper_returns_all_lands() {
    let mut g = pod(2);
    let ws = g.add_card_to_battlefield(0, catalog::world_shaper());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(ws))).expect("bolt");
    let f = named(&g, 0, "Forest");
    assert_eq!(f.len(), 3);
    assert!(f.iter().all(|id| g.battlefield_find(*id).unwrap().tapped));
}

/// Winding Way takes the chosen type from the top four.
#[test]
fn winding_way_takes_lands() {
    let mut g = pod(2);
    for f in [catalog::forest, catalog::grizzly_bears, catalog::forest, catalog::lightning_bolt] {
        g.add_card_to_library(0, f());
    }
    let ww = g.add_card_to_hand(0, catalog::winding_way());
    let h = g.players[0].hand.len() - 1;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: ww, target: None, additional_targets: vec![], mode: Some(1), x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h + 2);
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name != "Winding Way").count(), 2);
}

/// Rumbleweed pumps your other creatures on entry.
#[test]
fn rumbleweed_pumps_the_team() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rw = g.add_card_to_hand(0, catalog::rumbleweed());
    cast(&mut g, 0, rw, None).expect("cast");
    assert_eq!(pt(&g, bear), (5, 5));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Trample));
}

/// Yuma entering may sacrifice a land to draw.
#[test]
fn yuma_trades_a_land_for_a_card() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    g.add_card_to_battlefield(0, catalog::forest());
    let y = g.add_card_to_hand(0, catalog::yuma_proud_protector());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let h = g.players[0].hand.len() - 1;
    cast(&mut g, 0, y, None).expect("cast");
    assert!(named(&g, 0, "Forest").is_empty());
    assert_eq!(g.players[0].hand.len(), h + 1);
}

/// Cactus Preserve becomes an X/X reach Plant, X your commander's mana value.
#[test]
fn cactus_preserve_animates() {
    let mut g = pod(2);
    let cp = g.add_card_to_battlefield(0, catalog::cactus_preserve());
    let cmd = g.add_card_to_battlefield(0, catalog::yuma_proud_protector());
    g.players[0].commanders.push(cmd);
    activate(&mut g, 0, cp, 1).expect("animate");
    let c = g.computed_permanent(cp).unwrap();
    assert_eq!((c.power, c.toughness), (8, 8));
    assert!(c.keywords().contains(&Keyword::Reach));
}

/// Shefet Dunes sacrifices a Desert (itself) for +1/+1 on your creatures.
#[test]
fn shefet_dunes_pumps() {
    let mut g = pod(2);
    let sd = g.add_card_to_battlefield(0, catalog::shefet_dunes());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, sd, 2).expect("pump");
    assert_eq!(pt(&g, bear), (3, 3));
    assert!(g.battlefield_find(sd).is_none());
}

/// Perpetual Timepiece mills two.
#[test]
fn perpetual_timepiece_mills() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let pt_ = g.add_card_to_battlefield(0, catalog::perpetual_timepiece());
    let gy = g.players[0].graveyard.len();
    activate(&mut g, 0, pt_, 0).expect("mill");
    assert_eq!(g.players[0].graveyard.len(), gy + 2);
}

/// Embrace the Unknown exiles two playable cards.
#[test]
fn embrace_the_unknown_exiles_two() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let e = g.add_card_to_hand(0, catalog::embrace_the_unknown());
    cast(&mut g, 0, e, None).expect("cast");
    assert_eq!(g.exile.iter().filter(|c| c.may_play_until.is_some()).count(), 2);
}

/// Perennial Behemoth lets you play lands from your graveyard.
#[test]
fn perennial_behemoth_plays_graveyard_lands() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::perennial_behemoth());
    let f = g.add_card_to_graveyard(0, catalog::forest());
    g.perform_action(GameAction::PlayLandFromGraveyard(f)).expect("land from graveyard");
    assert_eq!(named(&g, 0, "Forest").len(), 1);
}

