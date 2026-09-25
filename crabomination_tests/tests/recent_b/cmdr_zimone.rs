//! Commander: the Jump Scare! precon (DSC, Zimone,
//! `decks::cmdr_zimone`).

use crabomination::card::{CardId, CardType, CounterType, Keyword};
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

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    act(g, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
}

fn face_down(g: &GameState, seat: usize) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.face_down).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn stock(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::grizzly_bears());
    }
}

fn to_step(g: &mut GameState, step: TurnStep) {
    for _ in 0..20 {
        if g.step == step {
            return;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    panic!("never reached {step:?}");
}

fn attack(g: &mut GameState, attacker: CardId) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(g);
}

/// Ashaya makes your nontoken creatures Forest lands and counts lands.
/// ⚠ Residual: its P/T counts printed lands only, not the creatures it made
/// lands (a CDA reading the layer-4 result would recurse into the layers).
#[test]
fn ashaya_turns_creatures_into_forests() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let a = g.add_card_to_battlefield(0, catalog::ashaya_soul_of_the_wild());
    assert_eq!(pt(&g, a), (3, 3), "the three Forests (residual: not the Bear or Ashaya)");
    assert!(g.computed_permanent(bear).unwrap().card_types().contains(&CardType::Land));
}

/// Curator Beastie manifests dread on entering, and the colorless 2/2 gets
/// two extra counters.
#[test]
fn curator_beastie_grows_its_manifests() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    let c = g.add_card_to_hand(0, catalog::curator_beastie());
    cast_at(&mut g, c, &[]).expect("cast");
    let fd = face_down(&g, 0);
    assert_eq!(fd.len(), 1);
    assert_eq!(pt(&g, fd[0]), (4, 4));
}

/// Deathmist Raptor comes back when a permanent of yours turns face up.
#[test]
fn deathmist_raptor_returns_on_a_turn_up() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    let raptor = g.add_card_to_graveyard(0, catalog::deathmist_raptor());
    let s = g.add_card_to_battlefield(0, catalog::scroll_of_fate());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    activate(&mut g, s, 0, &[]).expect("manifest from hand");
    assert!(g.battlefield_find(bear).is_some_and(|c| c.face_down));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    act(&mut g, GameAction::TurnFaceUp { card_id: bear }).expect("face up");
    assert!(g.battlefield_find(raptor).is_some());
}

/// Disorienting Choice: a kept permanent pays you a land.
#[test]
fn disorienting_choice_ramps_off_a_kept_permanent() {
    let mut g = main_phase(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    let d = g.add_card_to_hand(0, catalog::disorienting_choice());
    cast_at(&mut g, d, &[Target::Permanent(ring)]).expect("cast");
    assert!(g.battlefield_find(ring).is_some());
    assert!(g.battlefield_find(forest).is_some_and(|c| c.tapped));
}

/// Experimental Lab manifests dread with two counters and a trample counter.
#[test]
fn experimental_lab_builds_a_monster() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    let room = g.add_card_to_hand(0, catalog::experimental_lab_staff_room());
    act(&mut g, GameAction::CastRoomDoor { card_id: room, right: false }).expect("cast the Lab");
    let fd = face_down(&g, 0);
    assert_eq!(fd.len(), 1);
    assert_eq!(pt(&g, fd[0]), (4, 4));
    assert!(g.computed_permanent(fd[0]).unwrap().keywords().contains(&Keyword::Trample));
}

/// Giggling Skitterspike pings each opponent when it attacks.
#[test]
fn giggling_skitterspike_pings_on_attack() {
    let mut g = main_phase(3);
    let s = g.add_card_to_battlefield(0, catalog::giggling_skitterspike());
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    attack(&mut g, s);
    assert_eq!((g.players[1].life, g.players[2].life), (l1 - 1, l2 - 1));
}

/// Glitch Interpreter bounces itself and manifests dread with no face-down
/// permanent out.
#[test]
fn glitch_interpreter_glitches() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    let gi = g.add_card_to_hand(0, catalog::glitch_interpreter());
    cast_at(&mut g, gi, &[]).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == gi));
    assert_eq!(face_down(&g, 0).len(), 1);
}

/// Kefnet can't attack with a short hand; its ability draws.
#[test]
fn kefnet_needs_a_full_hand() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    let k = g.add_card_to_battlefield(0, catalog::kefnet_the_mindful());
    g.clear_sickness(k);
    assert!(g.computed_permanent(k).unwrap().keywords().contains(&Keyword::CantAttack));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    activate(&mut g, k, 0, &[]).expect("draw");
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Kheru Spellsnatcher turned face up steals the spell on the stack.
#[test]
fn kheru_spellsnatcher_snatches() {
    let mut g = main_phase(2);
    let k = g.add_card_to_hand(0, catalog::kheru_spellsnatcher());
    act(&mut g, GameAction::CastFaceDown { card_id: k }).expect("face down");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("opponent bolts");
    let life = g.players[0].life;
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::TurnFaceUp { card_id: k }).expect("face up");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "the Bolt was countered");
    assert!(g.exile.iter().any(|c| c.id == bolt));
}

/// Kianne: even power, a noncreature spell at instant speed; each draw grows
/// it.
#[test]
fn kianne_flashes_by_parity() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    let k = g.add_card_to_battlefield(0, catalog::kianne_corrupted_memory());
    g.active_player_idx = 1;
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast_at(&mut g, ring, &[]).expect("power 2: noncreature flash");
    g.active_player_idx = 0;
    let bs = g.add_card_to_hand(0, catalog::divination());
    cast_at(&mut g, bs, &[]).expect("draw two");
    assert_eq!(g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Scroll of Fate manifests a card from hand.
#[test]
fn scroll_of_fate_manifests_from_hand() {
    let mut g = main_phase(2);
    let s = g.add_card_to_battlefield(0, catalog::scroll_of_fate());
    let c = g.add_card_to_hand(0, catalog::lightning_bolt());
    activate(&mut g, s, 0, &[]).expect("manifest");
    assert!(g.battlefield_find(c).is_some_and(|x| x.face_down));
}

/// Shriekwood Devourer untaps lands equal to the biggest attacker's power.
#[test]
fn shriekwood_devourer_untaps_lands() {
    let mut g = main_phase(2);
    let lands: Vec<CardId> = (0..3).map(|_| g.add_card_to_battlefield(0, catalog::forest())).collect();
    for &l in &lands {
        g.battlefield_find_mut(l).unwrap().tapped = true;
    }
    let s = g.add_card_to_battlefield(0, catalog::shriekwood_devourer());
    attack(&mut g, s);
    assert!(lands.iter().all(|&l| !g.battlefield_find(l).unwrap().tapped));
}

/// Skaab Ruinator casts from the graveyard, exiling three creature cards.
#[test]
fn skaab_ruinator_rises_from_the_graveyard() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let s = g.add_card_to_graveyard(0, catalog::skaab_ruinator());
    act(&mut g, GameAction::CastFlashback {
        card_id: s,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from the graveyard");
    assert!(g.battlefield_find(s).is_some());
    assert_eq!(g.players[0].graveyard.len(), 0);
}

/// Tangled Islet is a Forest Island that enters tapped.
#[test]
fn tangled_islet_enters_tapped() {
    let mut g = main_phase(2);
    let l = g.add_card_to_hand(0, catalog::tangled_islet());
    act(&mut g, GameAction::PlayLand(l)).expect("play");
    assert!(g.battlefield_find(l).unwrap().tapped);
}

/// Temur War Shaman manifests on entering; a creature turned up fights.
#[test]
fn temur_war_shaman_turns_up_fighting() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::temur_war_shaman());
    cast_at(&mut g, t, &[]).expect("cast");
    let fd = face_down(&g, 0);
    assert_eq!(fd.len(), 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    act(&mut g, GameAction::TurnFaceUp { card_id: fd[0] }).expect("face up");
    assert!(g.battlefield_find(foe).is_none(), "the Bears traded");
}

/// Trail of Mystery: a face-down creature fetches a basic; a turn-up pumps.
#[test]
fn trail_of_mystery_rewards_mystery() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::trail_of_mystery());
    let s = g.add_card_to_battlefield(0, catalog::scroll_of_fate());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    activate(&mut g, s, 0, &[]).expect("manifest");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"));
    act(&mut g, GameAction::TurnFaceUp { card_id: bear }).expect("face up");
    assert_eq!(pt(&g, bear), (4, 4));
}

/// They Came from the Pipes manifests dread twice and draws for each.
#[test]
fn they_came_from_the_pipes_draws_per_manifest() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 6);
    let p = g.add_card_to_hand(0, catalog::they_came_from_the_pipes());
    cast_at(&mut g, p, &[]).expect("cast");
    assert_eq!(face_down(&g, 0).len(), 2);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Whisperwood Elemental manifests at your end step.
#[test]
fn whisperwood_elemental_manifests_at_end_step() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    g.add_card_to_battlefield(0, catalog::whisperwood_elemental());
    to_step(&mut g, TurnStep::End);
    assert_eq!(face_down(&g, 0).len(), 1);
}

/// Zimone's Hypothesis: a counter makes the Bears odd, and odd goes home
/// (CR 208.1 — zero is even).
#[test]
fn zimones_hypothesis_bounces_a_parity() {
    let mut g = main_phase(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let h = g.add_card_to_hand(0, catalog::zimones_hypothesis());
    cast_at(&mut g, h, &[Target::Permanent(mine)]).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "3/3: odd");
    assert!(g.battlefield_find(theirs).is_some(), "2/2: even");
}

/// Zimone: the first land each turn manifests dread, the second may turn a
/// face-down creature up.
#[test]
fn zimone_manifests_then_reveals() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 4);
    g.add_card_to_battlefield(0, catalog::zimone_mystery_unraveler());
    let a = g.add_card_to_hand(0, catalog::forest());
    act(&mut g, GameAction::PlayLand(a)).expect("land one");
    let fd = face_down(&g, 0);
    assert_eq!(fd.len(), 1);
    g.players[0].lands_played_this_turn = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let b = g.add_card_to_hand(0, catalog::island());
    act(&mut g, GameAction::PlayLand(b)).expect("land two");
    assert!(g.battlefield_find(fd[0]).is_some_and(|c| !c.face_down));
}
