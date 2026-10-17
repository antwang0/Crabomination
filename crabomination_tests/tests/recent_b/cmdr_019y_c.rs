//! Commander: the 2026-10-17 (`019Y6J4R`) third closer batch
//! (`decks::cmdr_019y_c`).

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
    g.players[seat].mana_pool.add_colorless(6);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    cast_mode(g, id, target, None)
}

fn cast_mode(
    g: &mut GameState,
    id: CardId,
    target: Option<Target>,
    mode: Option<usize>,
) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode,
        x_value: None,
    })?;
    drain_stack(g);
    Ok(())
}







fn attack_unblocked(g: &mut GameState, attacker: CardId, defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(defender),
    }]))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// Clattering Skeletons ventures as it dies.
#[test]
fn clattering_skeletons_ventures_on_death() {
    let mut g = pod(2);
    let s = ready(&mut g, 0, catalog::clattering_skeletons());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    cast(&mut g, bolt, Some(Target::Permanent(s))).expect("bolt");
    assert!(g.players[0].dungeon.is_some(), "entered a dungeon");
}

/// Starnheim Unleashed makes one Angel hard-cast and X foretold (CR 702.143).
#[test]
fn starnheim_unleashed_makes_x_angels_foretold() {
    let mut g = pod(2);
    let su = g.add_card_to_hand(0, catalog::starnheim_unleashed());
    flood(&mut g, 0);
    cast(&mut g, su, None).expect("cast");
    assert_eq!(named(&g, 0, "Angel Warrior"), 1);
    let su = g.add_card_to_hand(0, catalog::starnheim_unleashed());
    flood(&mut g, 0);
    g.perform_action(GameAction::Foretell { card_id: su }).expect("foretell");
    g.foretold_this_turn.clear();
    g.perform_action(GameAction::CastForetold {
        card_id: su,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("cast foretold for X = 3");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Angel Warrior"), 4);
}

/// Barrowin returns a small creature card attacking, but only once you've
/// completed a dungeon.
#[test]
fn barrowin_reanimates_only_after_a_dungeon() {
    let mut g = pod(2);
    let bw = ready(&mut g, 0, catalog::barrowin_of_clan_undurr());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    attack_unblocked(&mut g, bw, 1);
    assert!(g.battlefield_find(bears).is_none(), "no dungeon completed");
    g.players[0].dungeons_completed = 1;
    g.battlefield_find_mut(bw).unwrap().tapped = false;
    g.active_player_idx = 0;
    attack_unblocked(&mut g, bw, 1);
    assert!(g.battlefield_find(bears).is_some());
}

/// Seraphic Greatsword's wearer attacking the life leader brings an attacking
/// Angel; attacking someone lower doesn't.
#[test]
fn seraphic_greatsword_angels_only_at_the_life_leader() {
    let mut g = pod(3);
    let sword = ready(&mut g, 0, catalog::seraphic_greatsword());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: sword, target: bear }).expect("equip");
    drain_stack(&mut g);
    g.players[1].life = 30;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(2) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Angel"), 0, "seat 2 isn't the life leader");

    let mut g = pod(3);
    let sword = ready(&mut g, 0, catalog::seraphic_greatsword());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: sword, target: bear }).expect("equip");
    drain_stack(&mut g);
    g.players[1].life = 30;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let angel = g.battlefield.iter().find(|c| c.definition.name == "Angel").expect("an Angel");
    assert!(angel.tapped);
    assert_eq!(g.attack_for(angel.id).map(|a| a.target), Some(AttackTarget::Player(1)));
}

/// Varis ventures once a turn however many creature spells you cast.
#[test]
fn varis_ventures_once_each_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::varis_silverymoon_ranger());
    let a = g.add_card_to_hand(0, catalog::grizzly_bears());
    let b = g.add_card_to_hand(0, catalog::savannah_lions());
    flood(&mut g, 0);
    cast(&mut g, a, None).expect("cast");
    let room = g.players[0].dungeon.clone();
    assert!(room.is_some());
    cast(&mut g, b, None).expect("cast");
    assert_eq!(g.players[0].dungeon, room, "only once each turn");
}

/// Black Widow, Double Agent: a lone attacker gets first strike and menace.
#[test]
fn black_widow_double_agent_arms_a_lone_attacker() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::black_widow_double_agent());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords().contains(&Keyword::FirstStrike) && cp.keywords().contains(&Keyword::Menace));
}

/// Timecrafting adds X time counters to a permanent that has one.
#[test]
fn timecrafting_adds_x_time_counters() {
    let mut g = pod(2);
    let rock = ready(&mut g, 0, catalog::sol_ring());
    g.battlefield_find_mut(rock).unwrap().add_counters(CounterType::Time, 1);
    let tc = g.add_card_to_hand(0, catalog::timecrafting());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: tc,
        target: Some(Target::Permanent(rock)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: Some(3),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rock).unwrap().counter_count(CounterType::Time), 4);
}

/// Archfiend's Vessel hard-cast stays; reanimated, it becomes a 5/5 Demon.
#[test]
fn archfiends_vessel_turns_into_a_demon_from_the_graveyard() {
    let mut g = pod(2);
    let v = g.add_card_to_hand(0, catalog::archfiends_vessel());
    flood(&mut g, 0);
    cast(&mut g, v, None).expect("cast");
    assert!(g.battlefield_find(v).is_some());
    assert_eq!(named(&g, 0, "Demon"), 0);
    let v2 = g.add_card_to_graveyard(0, catalog::archfiends_vessel());
    let raise = g.add_card_to_hand(0, catalog::zombify());
    cast(&mut g, raise, Some(Target::Permanent(v2))).expect("reanimate");
    assert!(g.exile.iter().any(|c| c.id == v2));
    assert_eq!(named(&g, 0, "Demon"), 1);
}

/// Pizza Face's disappear turns a noncreature artifact into a 3/3 Mutant.
#[test]
fn pizza_face_animates_an_artifact_after_a_departure() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::pizza_face_gastromancer());
    let rock = ready(&mut g, 0, catalog::sol_ring());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("a permanent leaves");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let cp = g.computed_permanent(rock).expect("still here");
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Krang & Shredder exiles from each opponent's library until a nonland card,
/// linked to itself.
#[test]
fn krang_and_shredder_digs_each_opponent() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 1..3 {
        g.add_card_to_library(seat, catalog::forest());
        g.add_card_to_library(seat, catalog::grizzly_bears());
    }
    let ks = g.add_card_to_hand(0, catalog::krang_and_shredder());
    flood(&mut g, 0);
    cast(&mut g, ks, None).expect("cast");
    let linked = g.exile.iter().filter(|c| c.exiled_with == Some(ks)).count();
    assert_eq!(linked, 4, "a Forest and a Bears from each of two opponents");
}

/// Curse of Hospitality: a creature hitting the cursed player exiles their top
/// card, playable by the attacker this turn.
#[test]
fn curse_of_hospitality_hands_the_attacker_a_card() {
    let mut g = multi_player_game(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let top = g.add_card_to_library(1, catalog::grizzly_bears());
    let curse = g.add_card_to_hand(0, catalog::curse_of_hospitality());
    let bear = ready(&mut g, 0, catalog::savannah_lions());
    flood(&mut g, 0);
    cast(&mut g, curse, Some(Target::Player(1))).expect("curse");
    attack_unblocked(&mut g, bear, 1);
    let c = g.exile.iter().find(|c| c.id == top).expect("exiled");
    assert!(c.may_play_until.is_some_and(|m| m.player == 0));
}

/// Rankle and Torbran's +2 mode adds to every source's damage to a player.
#[test]
fn rankle_and_torbran_add_two_to_player_damage() {
    let mut g = pod(2);
    let rt = ready(&mut g, 0, catalog::rankle_and_torbran());
    attack_unblocked(&mut g, rt, 1);
    // The headless pick takes every mode; first strike dealt 3, then +2 was
    // armed for the rest of the turn.
    assert_eq!(g.players[1].life, 17);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    g.step = TurnStep::PostCombatMain;
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    assert_eq!(g.players[1].life, 12, "3 + 2");
}

/// Paradox Shaper prepares itself at upkeep and casts Omit Variables.
#[test]
fn paradox_shaper_prepares_and_mills() {
    let mut g = pod(2);
    let ps = ready(&mut g, 0, catalog::paradox_shaper());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ps).unwrap().counter_count(CounterType::Prepared), 1);
    g.step = TurnStep::PreCombatMain;
    flood(&mut g, 0);
    let lib = g.players[0].library.len();
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: ps,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the prepared spell");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 3);
}

/// Konstrari Improviser enters prepared; Soul Tether makes a mana Heartwood.
#[test]
fn konstrari_improviser_makes_a_heartwood() {
    let mut g = pod(2);
    let ki = g.add_card_to_hand(0, catalog::konstrari_improviser());
    flood(&mut g, 0);
    cast(&mut g, ki, None).expect("cast");
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: ki,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("enters prepared");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Heartwood"), 1);
}
