//! Commander: the World Shaper precon (EOC, Hearthhull,
//! `decks::cmdr_hearthhull`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    act(g, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    act(g, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn has(g: &GameState, id: CardId, k: &Keyword) -> bool {
    g.computed_permanent(id).expect("on the battlefield").keywords().contains(k)
}

fn attack_with(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for a in attackers {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn in_graveyard(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].graveyard.iter().any(|c| c.id == id)
}

/// CR 721 — Hearthhull at 8+ charge counters is a 6/7 flier with haste;
/// its 2+ ability sacrifices a land for two cards, and each land sacrifice
/// drains each opponent for 2.
#[test]
fn cr_721_hearthhull_stations_and_sacrifices_lands() {
    let mut g = main_phase(3);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    let hh = g.add_card_to_battlefield(0, catalog::hearthhull_the_worldseed());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(hh).unwrap().add_counters(CounterType::Charge, 8);
    assert_eq!(pt(&g, hh), (6, 7));
    assert!(has(&g, hh, &Keyword::Flying) && has(&g, hh, &Keyword::Haste));
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    activate(&mut g, hh, 1, None).expect("2+ ability");
    assert!(in_graveyard(&g, 0, land));
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!((g.players[1].life, g.players[2].life), (l1 - 2, l2 - 2));
}

/// The SNC sacrifice lands: sacrifice on entry for a tapped basic of one of
/// three types, and 1 life.
#[test]
fn courtyard_and_theater_fetch_a_tapped_basic() {
    for (def, basic) in [
        (catalog::cabaretti_courtyard as fn() -> _, catalog::mountain as fn() -> _),
        (catalog::maestros_theater, catalog::swamp),
    ] {
        let mut g = main_phase(2);
        let b = g.add_card_to_library(0, basic());
        let land = g.add_card_to_hand(0, def());
        let life = g.players[0].life;
        act(&mut g, GameAction::PlayLand(land)).expect("play");
        assert!(in_graveyard(&g, 0, land));
        assert!(g.battlefield_find(b).is_some_and(|c| c.tapped));
        assert_eq!(g.players[0].life, life + 1);
    }
}

/// CR 603.10 — Eumidian Hatchery counts its hatchling counters as it leaves.
#[test]
fn cr_603_10_eumidian_hatchery_hatches_per_counter() {
    let mut g = main_phase(2);
    let h = g.add_card_to_battlefield(0, catalog::eumidian_hatchery());
    let life = g.players[0].life;
    activate(&mut g, h, 0, None).expect("tap for B");
    assert_eq!(g.players[0].life, life - 1);
    g.battlefield_find_mut(h).unwrap().tapped = false;
    activate(&mut g, h, 0, None).expect("tap again");
    let rain = g.add_card_to_hand(0, catalog::stone_rain());
    cast_at(&mut g, rain, &[Target::Permanent(h)]).expect("stone rain");
    let insects = named(&g, 0, "Insect");
    assert_eq!(insects.len(), 2);
    assert!(has(&g, insects[0], &Keyword::Flying));
}

/// Eumidian Wastewaker's attack makes both players discard, drawing per land
/// discarded.
#[test]
fn eumidian_wastewaker_discards_both_sides() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(1, catalog::island());
    let w = g.add_card_to_battlefield(0, catalog::eumidian_wastewaker());
    attack_with(&mut g, &[w], 1);
    assert!(g.players[1].hand.is_empty());
    assert_eq!(g.players[0].hand.len(), 2, "two lands discarded, two drawn");
}

/// Festering Thicket is a Swamp Forest tapland with cycling; Horizon
/// Explorer untaps it and makes a Lander when you attack.
#[test]
fn horizon_explorer_untaps_lands_and_makes_landers() {
    let mut g = main_phase(2);
    let ft = g.add_card_to_hand(0, catalog::festering_thicket());
    assert!(catalog::festering_thicket().keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));
    let hx = g.add_card_to_battlefield(0, catalog::horizon_explorer());
    act(&mut g, GameAction::PlayLand(ft)).expect("play");
    assert!(!g.battlefield_find(ft).unwrap().tapped, "lands enter untapped");
    attack_with(&mut g, &[hx], 1);
    assert_eq!(named(&g, 0, "Lander").len(), 1);
}

/// Juri grows per sacrifice and deals its power when it dies.
#[test]
fn juri_grows_on_sacrifice_and_burns_on_death() {
    let mut g = main_phase(2);
    let juri = g.add_card_to_battlefield(0, catalog::juri_master_of_the_revue());
    let uurg = g.add_card_to_battlefield(0, catalog::uurg_spawn_of_turg());
    g.add_card_to_battlefield(0, catalog::forest());
    activate(&mut g, uurg, 0, None).expect("sacrifice a land");
    assert_eq!(pt(&g, juri), (2, 2));
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    let kill = g.add_card_to_hand(0, catalog::murder());
    cast_at(&mut g, kill, &[Target::Permanent(juri)]).expect("murder");
    assert_eq!(g.players[1].life, life - 2);
}

/// Loamcrafter Faun trades discarded lands for as many permanent cards back.
#[test]
fn loamcrafter_faun_trades_lands_for_permanents() {
    use crabomination::game::effects::EffectContext;
    let mut g = main_phase(2);
    let f1 = g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![f1])]));
    let etb = catalog::loamcrafter_faun().triggered_abilities[0].effect.clone();
    let ctx = EffectContext {
        targets: vec![Target::Permanent(bear), Target::Permanent(angel)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&etb, &ctx).unwrap();
    assert!(in_graveyard(&g, 0, f1));
    let back = [bear, angel].iter().filter(|id| g.players[0].hand.iter().any(|c| c.id == **id)).count();
    assert_eq!(back, 1, "one land discarded, one card back");
}

/// Moraug: landfall in your main phase adds a combat; an attacker gets +1/+0.
#[test]
fn moraug_landfall_adds_a_combat() {
    let mut g = main_phase(2);
    let m = g.add_card_to_battlefield(0, catalog::moraug_fury_of_akoum());
    let land = g.add_card_to_hand(0, catalog::forest());
    act(&mut g, GameAction::PlayLand(land)).expect("play");
    assert_eq!(g.additional_post_main_combats, 1);
    attack_with(&mut g, &[m], 1);
    assert_eq!(pt(&g, m), (7, 6));
}

/// Planetary Annihilation leaves six lands a player and 6 damage everywhere.
#[test]
fn planetary_annihilation_keeps_six_lands() {
    let mut g = main_phase(2);
    for _ in 0..8 {
        g.add_card_to_battlefield(1, catalog::forest());
    }
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let pa = g.add_card_to_hand(0, catalog::planetary_annihilation());
    cast_at(&mut g, pa, &[]).expect("cast");
    assert_eq!(named(&g, 1, "Forest").len(), 6);
    assert!(g.battlefield_find(angel).is_none());
}

/// Scouring Swarm: a tapped Insect per land sacrificed, or a tapped copy once
/// seven lands are in your graveyard.
#[test]
fn scouring_swarm_breeds_on_land_sacrifice() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::scouring_swarm());
    let uurg = g.add_card_to_battlefield(0, catalog::uurg_spawn_of_turg());
    g.add_card_to_battlefield(0, catalog::forest());
    activate(&mut g, uurg, 0, None).expect("sacrifice");
    let insects = named(&g, 0, "Insect");
    assert_eq!(insects.len(), 1);
    assert!(g.battlefield_find(insects[0]).unwrap().tapped);
    for _ in 0..6 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    g.add_card_to_battlefield(0, catalog::forest());
    activate(&mut g, uurg, 0, None).expect("sacrifice");
    assert_eq!(named(&g, 0, "Scouring Swarm").len(), 2, "a copy at seven lands");
}

/// Soul of Windgrace steals a land from a graveyard; its discard abilities.
#[test]
fn soul_of_windgrace_takes_a_graveyard_land() {
    let mut g = main_phase(2);
    let land = g.add_card_to_graveyard(1, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let sw = g.add_card_to_hand(0, catalog::soul_of_windgrace());
    cast_at(&mut g, sw, &[]).expect("cast");
    let l = g.battlefield_find(land).expect("the land came over");
    assert!(l.tapped && l.controller == 0);
    g.add_card_to_hand(0, catalog::forest());
    let life = g.players[0].life;
    activate(&mut g, sw, 0, None).expect("discard a land");
    assert_eq!(g.players[0].life, life + 3);
}

/// Sprouting Goblin kicked finds a basic-typed land.
#[test]
fn sprouting_goblin_kicked_finds_a_land() {
    let mut g = main_phase(2);
    let f = g.add_card_to_library(0, catalog::forest());
    let sg = g.add_card_to_hand(0, catalog::sprouting_goblin());
    act(&mut g, GameAction::CastSpellKicked { card_id: sg, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("kicked");
    assert!(g.players[0].hand.iter().any(|c| c.id == f));
}

/// Szarel plays lands from the graveyard, and a nontoken sacrifice on your
/// turn feeds another creature counters equal to Szarel's power.
#[test]
fn szarel_replays_lands_and_feeds_sacrifices() {
    let mut g = main_phase(2);
    let sz = g.add_card_to_battlefield(0, catalog::szarel_genesis_shepherd());
    let gy_land = g.add_card_to_graveyard(0, catalog::forest());
    act(&mut g, GameAction::PlayLandFromGraveyard(gy_land)).expect("play from graveyard");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let uurg = g.add_card_to_battlefield(0, catalog::uurg_spawn_of_turg());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    activate(&mut g, uurg, 0, None).expect("sacrifice the land");
    let _ = sz;
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// CR 604.3 — Uurg's power counts the lands in your graveyard; upkeep
/// surveil.
#[test]
fn cr_604_3_uurg_counts_graveyard_lands() {
    let mut g = main_phase(2);
    let u = g.add_card_to_battlefield(0, catalog::uurg_spawn_of_turg());
    assert_eq!(pt(&g, u), (0, 5));
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    assert_eq!(pt(&g, u), (3, 5));
}

/// Windgrace's Judgment destroys one nonland permanent per opponent.
#[test]
fn windgraces_judgment_hits_each_opponent() {
    let mut g = main_phase(3);
    let a = g.add_card_to_battlefield(1, catalog::serra_angel());
    let b = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let wj = g.add_card_to_hand(0, catalog::windgraces_judgment());
    cast_at(&mut g, wj, &[Target::Permanent(a), Target::Permanent(b)]).expect("cast");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}
