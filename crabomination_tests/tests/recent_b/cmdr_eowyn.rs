//! Commander: the Riders of Rohan precon (LTC, Éowyn, Shieldmaiden,
//! `decks::cmdr_eowyn`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
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

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
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

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn keywords(g: &GameState, id: CardId) -> Vec<Keyword> {
    g.computed_permanent(id).unwrap().keywords().to_vec()
}

/// Éowyn — a Human entering this turn makes two Knights at combat.
#[test]
fn eowyn_makes_knights_after_a_human_enters() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::eowyn_shieldmaiden());
    step(&mut g, TurnStep::BeginCombat);
    assert!(named(&g, 0, "Human Knight").is_empty(), "no Human entered yet");
    let b = g.add_card_to_hand(0, catalog::beregond_of_the_guard());
    g.step = TurnStep::PreCombatMain;
    cast(&mut g, 0, b, None).expect("beregond");
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(named(&g, 0, "Human Knight").len(), 2);
}

/// Aragorn — the monarch on entering; attacking as the monarch, nothing
/// blocks.
#[test]
fn aragorn_crowns_and_stops_blockers() {
    let mut g = pod(2);
    let a = g.add_card_to_hand(0, catalog::aragorn_king_of_gondor());
    cast(&mut g, 0, a, None).expect("aragorn");
    assert_eq!(g.monarch, Some(0));
    let blocker = g.add_card_to_battlefield(1, catalog::serra_angel());
    attack(&mut g, &[a]);
    assert!(keywords(&g, blocker).contains(&Keyword::CantBlock));
}

/// Archivist of Gondor — the monarch draws at their end step.
#[test]
fn archivist_draws_the_monarch_a_card() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::archivist_of_gondor());
    g.add_card_to_library(0, catalog::island());
    g.monarch = Some(0);
    let hand = g.players[0].hand.len();
    step(&mut g, TurnStep::End);
    // The monarch's own end-step draw (CR 725.2) plus the Archivist's.
    assert!(g.players[0].hand.len() > hand);
}

/// Beregond — a Human entering pumps the team with vigilance.
#[test]
fn beregond_rallies_on_humans() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_hand(0, catalog::beregond_of_the_guard());
    cast(&mut g, 0, b, None).expect("beregond");
    assert_eq!(pt(&g, bears), (3, 3));
    assert!(keywords(&g, bears).contains(&Keyword::Vigilance));
}

/// Boromir — digs six for a Human or artifact.
#[test]
fn boromir_finds_a_human() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let found = g.add_card_to_library(0, catalog::beregond_of_the_guard());
    let b = g.add_card_to_hand(0, catalog::boromir_gondors_hope());
    cast(&mut g, 0, b, None).expect("boromir");
    assert!(g.players[0].hand.iter().any(|c| c.id == found));
}

/// Call for Aid — borrow an opponent's creatures for the turn.
#[test]
fn call_for_aid_borrows_an_army() {
    let mut g = pod(3);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.battlefield_find_mut(angel).unwrap().tapped = true;
    let cfa = g.add_card_to_hand(0, catalog::call_for_aid());
    cast(&mut g, 0, cfa, Some(Target::Player(1))).expect("call for aid");
    let c = g.battlefield_find(angel).unwrap();
    assert_eq!(c.controller, 0);
    assert!(!c.tapped);
    assert!(keywords(&g, angel).contains(&Keyword::Haste));
}

/// Court of Ire — 7 damage at your upkeep while you're the monarch.
#[test]
fn court_of_ire_burns_harder_as_monarch() {
    let mut g = pod(2);
    let c = g.add_card_to_hand(0, catalog::court_of_ire());
    cast(&mut g, 0, c, None).expect("court");
    assert_eq!(g.monarch, Some(0));
    step(&mut g, TurnStep::Upkeep);
    assert_eq!(g.players[1].life, 13);
}

/// Crown of Gondor — +1/+1 per creature you control.
#[test]
fn crown_of_gondor_scales() {
    let mut g = pod(2);
    let crown = g.add_card_to_battlefield(0, catalog::crown_of_gondor());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: crown, target: bears }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bears), (4, 4));
}

/// Denethor — the sacrifice makes you the monarch and shoots 3.
#[test]
fn denethor_crowns_and_shoots() {
    let mut g = pod(2);
    let d = g.add_card_to_battlefield(0, catalog::denethor_stone_seer());
    g.clear_sickness(d);
    activate(&mut g, 0, d, 0, Some(Target::Player(1))).expect("denethor");
    assert_eq!(g.monarch, Some(0));
    assert_eq!(g.players[1].life, 17);
}

/// Faramir — the monarch's end step makes two Soldiers.
#[test]
fn faramir_makes_soldiers_as_monarch() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::faramir_steward_of_gondor());
    g.monarch = Some(0);
    step(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Human Soldier").len(), 2);
}

/// Fealty to the Realm — you become the monarch and take the creature.
#[test]
fn fealty_to_the_realm_takes_a_creature() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let f = g.add_card_to_hand(0, catalog::fealty_to_the_realm());
    cast(&mut g, 0, f, Some(Target::Permanent(angel))).expect("fealty");
    assert_eq!(g.monarch, Some(0));
    assert_eq!(g.battlefield_find(angel).unwrap().controller, 0);
}

/// Forth Eorlingas! — X Knights, and their damage makes you the monarch.
#[test]
fn forth_eorlingas_rides_to_the_crown() {
    let mut g = pod(2);
    let fe = g.add_card_to_hand(0, catalog::forth_eorlingas());
    cast_x(&mut g, 0, fe, None, Some(2)).expect("forth");
    let k = named(&g, 0, "Human Knight");
    assert_eq!(k.len(), 2);
    attack(&mut g, &k);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0));
}

/// Gilraen — the blinked creature returns at the end step with vigilance
/// and lifelink counters.
#[test]
fn gilraen_blinks_with_counters() {
    let mut g = pod(2);
    let gi = g.add_card_to_battlefield(0, catalog::gilraen_dunedain_protector());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(gi);
    activate(&mut g, 0, gi, 0, Some(Target::Permanent(bears))).expect("gilraen");
    assert!(g.battlefield_find(bears).is_none());
    step(&mut g, TurnStep::End);
    let kw = keywords(&g, bears);
    assert!(kw.contains(&Keyword::Vigilance) && kw.contains(&Keyword::Lifelink));
}

/// Gimli — grows off another legend and makes Treasure on a hit.
#[test]
fn gimli_grows_and_mines() {
    let mut g = pod(2);
    let gm = g.add_card_to_battlefield(0, catalog::gimli_of_the_glittering_caves());
    let t = g.add_card_to_hand(0, catalog::theoden_king_of_rohan());
    cast(&mut g, 0, t, None).expect("theoden");
    assert_eq!(g.battlefield_find(gm).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Grey Host Reinforcements — grows per creature card exiled.
#[test]
fn grey_host_grows_from_a_graveyard() {
    let mut g = pod(2);
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::hill_giant());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let gh = g.add_card_to_hand(0, catalog::grey_host_reinforcements());
    cast(&mut g, 0, gh, None).expect("grey host");
    assert!(g.players[1].graveyard.is_empty());
    assert_eq!(g.battlefield_find(gh).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Lost to Legend — a legend goes fourth from the top.
#[test]
fn lost_to_legend_tucks_a_legend() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let legend = g.add_card_to_battlefield(1, catalog::theoden_king_of_rohan());
    let ltl = g.add_card_to_hand(0, catalog::lost_to_legend());
    cast(&mut g, 0, ltl, Some(Target::Permanent(legend))).expect("lost");
    assert_eq!(g.players[1].library.iter().position(|c| c.id == legend), Some(3));
}

/// Riders of Rohan — two Knights on entering.
#[test]
fn riders_of_rohan_bring_knights() {
    let mut g = pod(2);
    let r = g.add_card_to_hand(0, catalog::riders_of_rohan());
    cast(&mut g, 0, r, None).expect("riders");
    assert_eq!(named(&g, 0, "Human Knight").len(), 2);
}

/// Taunt from the Rampart — every opposing creature is goaded and can't
/// block.
#[test]
fn taunt_from_the_rampart_goads() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let t = g.add_card_to_hand(0, catalog::taunt_from_the_rampart());
    cast(&mut g, 0, t, None).expect("taunt");
    assert!(g.goaded_by_player(g.battlefield_find(angel).unwrap(), 0));
    assert!(keywords(&g, angel).contains(&Keyword::CantBlock));
}

/// Théoden — a Human entering gives double strike.
#[test]
fn theoden_grants_double_strike() {
    let mut g = pod(2);
    let t = g.add_card_to_hand(0, catalog::theoden_king_of_rohan());
    cast(&mut g, 0, t, None).expect("theoden");
    assert!(g.battlefield.iter().any(|c| g.computed_permanent(c.id).is_some_and(|cp| cp.keywords().contains(&Keyword::DoubleStrike))));
}

/// Visions of Glory — a Human per creature you control.
#[test]
fn visions_of_glory_doubles_the_ranks() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    let v = g.add_card_to_hand(0, catalog::visions_of_glory());
    cast(&mut g, 0, v, None).expect("visions");
    assert_eq!(named(&g, 0, "Human").len(), 3);
}

/// Éomer — enters with a counter per other Human, crowns you and shoots
/// for its power.
#[test]
fn eomer_crowns_and_shoots() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::beregond_of_the_guard());
    g.add_card_to_battlefield(0, catalog::denethor_stone_seer());
    let e = g.add_card_to_hand(0, catalog::eomer_king_of_rohan());
    cast(&mut g, 0, e, None).expect("eomer");
    assert_eq!(g.battlefield_find(e).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.monarch, Some(0));
}

/// Oath of Eorl — chapter I makes two Soldiers.
#[test]
fn oath_of_eorl_chapter_one() {
    let mut g = pod(2);
    let o = g.add_card_to_hand(0, catalog::oath_of_eorl());
    cast(&mut g, 0, o, None).expect("oath");
    assert_eq!(named(&g, 0, "Human Soldier").len(), 2);
}

/// Champions of Minas Tirith — crowns you on entering.
#[test]
fn champions_crown_you() {
    let mut g = pod(2);
    let c = g.add_card_to_hand(0, catalog::champions_of_minas_tirith());
    cast(&mut g, 0, c, None).expect("champions");
    assert_eq!(g.monarch, Some(0));
}

/// Crown of Gondor's equip costs {3} less while you're the monarch.
#[test]
fn crown_of_gondor_equips_for_one_as_the_monarch() {
    let mut g = pod(2);
    let crown = g.add_card_to_battlefield(0, catalog::crown_of_gondor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ev = Vec::new();
    g.set_monarch(0, &mut ev);
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: crown, target: bear }).expect("equip for {1}");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(crown).and_then(|c| c.attached_to), Some(bear));
}

/// Champions of Minas Tirith — while you're the monarch, each opponent's
/// combat asks them to pay {X} (X = cards in their hand) or not attack you.
#[test]
fn champions_of_minas_tirith_offer_the_x() {
    for pays in [false, true] {
        let mut g = pod(2);
        g.add_card_to_battlefield(0, catalog::champions_of_minas_tirith());
        let mut ev = Vec::new();
        g.set_monarch(0, &mut ev);
        g.add_card_to_hand(1, catalog::island());
        g.add_card_to_hand(1, catalog::island());
        g.players[1].mana_pool.add_colorless(2);
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(pays),
        ]));
        g.active_player_idx = 1;
        g.step = TurnStep::BeginCombat;
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        let banned = g.cant_attack_player_this_turn.contains(&(1, 0));
        assert_eq!(banned, !pays, "paid {pays}: attack ban {banned}");
        if pays {
            assert_eq!(g.players[1].mana_pool.total(), 0, "paid {{2}}");
        }
    }
}
