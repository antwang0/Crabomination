//! Commander: the Hatsune Miku precon (SLD, Trostani, `decks::cmdr_trostani`).

use crabomination::card::{CardId, CounterType, Keyword};
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

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
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

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
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

/// A two-colored spell gains 2; the second colored spell that turn gains
/// nothing — once each turn.
#[test]
fn ancient_cornucopia_gains_per_color_once_a_turn() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::ancient_cornucopia());
    // "You may gain" — the seat takes it both times it is offered.
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let life = g.players[0].life;
    let rhys = g.add_card_to_hand(0, catalog::rhys_the_redeemed());
    cast(&mut g, 0, rhys, None).expect("cast");
    assert_eq!(g.players[0].life, life + 2, "green and white");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, 0, bear, None).expect("cast");
    assert_eq!(g.players[0].life, life + 2, "once each turn");
}

/// Angel of Indemnity returns a permanent card with mana value 4 or less;
/// its encore makes one hasty attacking copy per opponent (CR 702.141).
#[test]
fn angel_of_indemnity_recurs_and_encores_per_opponent() {
    let mut g = pod(3);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let angel = g.add_card_to_hand(0, catalog::angel_of_indemnity());
    cast(&mut g, 0, angel, None).expect("cast");
    assert!(g.battlefield_find(bear).is_some(), "the Bears came back");

    let mut g = pod(3);
    let angel = g.add_card_to_graveyard(0, catalog::angel_of_indemnity());
    activate(&mut g, 0, angel, 0, None, None).expect("encore");
    let copies = named(&g, 0, "Angel of Indemnity");
    assert_eq!(copies.len(), 2, "one per opponent");
    assert!(g.computed_permanent(copies[0]).unwrap().keywords().contains(&Keyword::Haste));
}

/// Blossoming Bogbeast's attack gains 2, then pumps the team by all the life
/// gained this turn — earlier gains included — and grants trample.
#[test]
fn blossoming_bogbeast_pumps_by_life_gained_this_turn() {
    let mut g = pod(2);
    let beast = g.add_card_to_battlefield(0, catalog::blossoming_bogbeast());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].life_gained_this_turn = 1;
    attack_with(&mut g, &[beast], 1);
    assert_eq!(pt(&g, bear), (5, 5), "+3/+3: one earlier, two now");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Trample));
}

/// Break Down destroys an artifact and leaves a Junk token, which exiles the
/// top card at sorcery speed for you to play this turn.
#[test]
fn break_down_leaves_junk_that_impulses() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let bd = g.add_card_to_hand(0, catalog::break_down());
    cast(&mut g, 0, bd, Some(Target::Permanent(ring))).expect("cast");
    assert!(g.battlefield_find(ring).is_none());
    let junk = named(&g, 0, "Junk");
    assert_eq!(junk.len(), 1);
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    activate(&mut g, 0, junk[0], 0, None, None).expect("sac the Junk");
    assert!(g.exile.iter().any(|c| c.id == top && c.may_play_until.is_some()), "exiled, playable this turn");
}

/// Brokers Hideout fetches a basic Island tapped and gains 1 life.
#[test]
fn brokers_hideout_fetches_a_brokers_basic() {
    let mut g = pod(2);
    let island = g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::mountain());
    let life = g.players[0].life;
    let hideout = g.add_card_to_hand(0, catalog::brokers_hideout());
    g.perform_action(GameAction::PlayLand(hideout)).expect("land");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hideout).is_none(), "sacrificed");
    assert!(g.battlefield_find(island).is_some_and(|c| c.tapped));
    assert_eq!(g.players[0].life, life + 1);
}

/// Conclave Evangelist copies itself on combat damage to a player; at three
/// seats myriad also sends a copy at the other opponent (CR 702.116a).
#[test]
fn conclave_evangelist_copies_itself_on_damage() {
    let mut g = pod(3);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let ev = g.add_card_to_battlefield(0, catalog::conclave_evangelist());
    let life = g.players[1].life;
    attack_with(&mut g, &[ev], 1);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, life - 4);
    assert_eq!(g.players[2].life, life - 4, "the myriad copy hit the other seat");
    // Each connecting Evangelist made a copy; the myriad token itself is gone.
    assert_eq!(named(&g, 0, "Conclave Evangelist").len(), 3);
}

/// Ghalta and Mavren's first mode makes an attacking Dinosaur as big as the
/// greatest power among the *other* attackers.
#[test]
fn ghalta_and_mavren_dinosaur_matches_the_biggest_other_attacker() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(0)]));
    let gm = g.add_card_to_battlefield(0, catalog::ghalta_and_mavren());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    attack_with(&mut g, &[gm, giant], 1);
    let dino = named(&g, 0, "Dinosaur");
    assert_eq!(dino.len(), 1);
    assert_eq!(pt(&g, dino[0]), (3, 3), "Hill Giant, not the 12/12");
    assert!(g.attacking.iter().any(|a| a.attacker == dino[0]));

    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(1)]));
    let gm = g.add_card_to_battlefield(0, catalog::ghalta_and_mavren());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack_with(&mut g, &[gm, a, b], 1);
    assert_eq!(named(&g, 0, "Vampire").len(), 2, "one per other attacker");
}

/// Grand Crescendo makes X Citizens and your creatures indestructible.
#[test]
fn grand_crescendo_makes_x_citizens_and_indestructible() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gc = g.add_card_to_hand(0, catalog::grand_crescendo());
    cast_x(&mut g, 0, gc, None, Some(3)).expect("cast");
    assert_eq!(named(&g, 0, "Citizen").len(), 3);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Indestructible));
}

/// Halo Fountain's cost untaps your tapped creatures; with only one tapped,
/// the two-creature draw can't be paid.
#[test]
fn halo_fountain_untaps_creatures_as_its_cost() {
    let mut g = pod(2);
    let hf = g.add_card_to_battlefield(0, catalog::halo_fountain());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let hand = g.players[0].hand.len();
    assert!(activate(&mut g, 0, hf, 1, None, None).is_err(), "two tapped creatures needed");
    assert!(g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(g.players[0].hand.len(), hand);
    activate(&mut g, 0, hf, 0, None, None).expect("one tapped creature");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped as the cost");
    assert_eq!(named(&g, 0, "Citizen").len(), 1);
}

/// Invincible Hymn sets your life to your library's size — up or down.
#[test]
fn invincible_hymn_sets_life_to_library_size() {
    let mut g = pod(2);
    while g.players[0].library.len() < 60 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.players[0].library.truncate(60);
    let hymn = g.add_card_to_hand(0, catalog::invincible_hymn());
    cast(&mut g, 0, hymn, None).expect("cast");
    assert_eq!(g.players[0].life, 60);
}

/// Lathiel puts +1/+1 counters, up to the life you gained this turn, on other
/// creatures at an end step; nothing without a gain.
#[test]
fn lathiel_distributes_the_life_gained_as_counters() {
    let mut g = pod(2);
    let lathiel = g.add_card_to_battlefield(0, catalog::lathiel_the_bounteous_dawn());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);

    g.players[0].life_gained_this_turn = 3;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    assert_eq!(g.battlefield_find(lathiel).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "other creatures");
}

/// Lazotep Quarry's third ability eternalizes a graveyard creature with mana
/// value X: a 4/4 black Zombie copy, the card exiled.
#[test]
fn lazotep_quarry_makes_a_four_four_black_zombie_copy() {
    let mut g = pod(2);
    let quarry = g.add_card_to_battlefield(0, catalog::lazotep_quarry());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    activate(&mut g, 0, quarry, 2, Some(Target::Permanent(bear)), Some(2)).expect("X = 2");
    assert!(g.battlefield_find(quarry).is_none(), "sacrificed as the Desert");
    assert!(g.exile.iter().any(|c| c.id == bear));
    let copy = named(&g, 0, "Grizzly Bears");
    assert_eq!(copy.len(), 1);
    assert_eq!(pt(&g, copy[0]), (4, 4));
    let colors = &g.computed_permanent(copy[0]).unwrap().colors;
    assert!(colors.contains(Color::Black) && colors.len() == 1, "{colors:?}");
}

/// Pest Infestation destroys up to X artifacts and makes 2X Pests.
#[test]
fn pest_infestation_destroys_x_and_makes_twice_x_pests() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(1, catalog::sol_ring());
    let b = g.add_card_to_battlefield(1, catalog::sol_ring());
    let pi = g.add_card_to_hand(0, catalog::pest_infestation());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: pi,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert_eq!(named(&g, 0, "Pest").len(), 4);
}

/// Rhys the Redeemed doubles your creature tokens and only yours.
#[test]
fn rhys_doubles_your_creature_tokens() {
    let mut g = pod(2);
    let rhys = g.add_card_to_battlefield(0, catalog::rhys_the_redeemed());
    g.clear_sickness(rhys);
    activate(&mut g, 0, rhys, 0, None, None).expect("an Elf Warrior");
    g.battlefield_find_mut(rhys).unwrap().tapped = false;
    activate(&mut g, 0, rhys, 0, None, None).expect("another");
    let opp = g.add_card_to_hand(1, catalog::grand_crescendo());
    cast_x(&mut g, 1, opp, None, Some(1)).expect("an opposing Citizen");
    g.battlefield_find_mut(rhys).unwrap().tapped = false;
    activate(&mut g, 0, rhys, 1, None, None).expect("copy them");
    assert_eq!(named(&g, 0, "Elf Warrior").len(), 4);
    assert_eq!(named(&g, 1, "Citizen").len(), 1);
    assert!(named(&g, 0, "Citizen").is_empty());
}

/// Sapseep Forest's life ability needs two green permanents.
#[test]
fn sapseep_forest_needs_two_green_permanents() {
    let mut g = pod(2);
    let forest = g.add_card_to_battlefield(0, catalog::sapseep_forest());
    g.battlefield_find_mut(forest).unwrap().tapped = false;
    let life = g.players[0].life;
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(activate(&mut g, 0, forest, 1, None, None).is_err(), "one green permanent");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, forest, 1, None, None).expect("two");
    assert_eq!(g.players[0].life, life + 1);
}

/// Song of Freyalise's chapter I grant lasts until your next turn (CR 611.2c:
/// the creatures you control as it resolves), and chapter III grows the team.
#[test]
fn song_of_freyalise_grant_ends_at_your_next_turn() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let song = g.add_card_to_hand(0, catalog::song_of_freyalise());
    cast(&mut g, 0, song, None).expect("cast");
    assert_eq!(g.battlefield_find(bear).unwrap().granted_activated_abilities.len(), 1, "chapter I");
    let late = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.battlefield_find(late).unwrap().granted_activated_abilities.is_empty(), "locked in");
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(1, catalog::forest());
    }
    advance_to(&mut g, TurnStep::End);
    while g.active_player_idx != 1 {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(bear).unwrap().granted_activated_abilities.len(), 1, "through their turn");
    while !(g.active_player_idx == 0 && g.step == TurnStep::Draw) {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(bear).unwrap().granted_activated_abilities.is_empty(), "ended at your next turn");
    advance_to(&mut g, TurnStep::PreCombatMain);
    assert_eq!(g.battlefield_find(bear).unwrap().granted_activated_abilities.len(), 1, "chapter II");
}

/// Song of Freyalise's chapter III: a counter and three keywords for each of
/// your creatures.
#[test]
fn song_of_freyalise_chapter_three_grows_the_team() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let song = g.add_card_to_battlefield(0, catalog::song_of_freyalise());
    g.battlefield_find_mut(song).unwrap().add_counters(CounterType::Lore, 2);
    g.saga_advance(song);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let kws = g.computed_permanent(bear).unwrap().keywords().to_vec();
    for k in [Keyword::Vigilance, Keyword::Trample, Keyword::Indestructible] {
        assert!(kws.contains(&k), "{k:?}");
    }
}

/// Soul of Eternity is as big as your life total.
#[test]
fn soul_of_eternity_tracks_your_life() {
    let mut g = pod(2);
    let soul = g.add_card_to_battlefield(0, catalog::soul_of_eternity());
    let life = g.players[0].life;
    assert_eq!(pt(&g, soul), (life, life));
    g.players[0].life = 7;
    assert_eq!(pt(&g, soul), (7, 7));
}

/// CR 117.3c — the activator gets priority back, so a bot could stack
/// Aetherflux Reservoir shots forever (a 6-seat pod held 5,486 of them at the
/// action cap). A bot seat lets its own shot resolve before firing again.
#[test]
fn bot_lets_its_own_aetherflux_shot_resolve_first() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = pod(3);
    for s in 0..3 {
        g.players[s].wants_ui = true;
    }
    g.players[0].life = 100_000;
    g.add_card_to_battlefield(0, catalog::aetherflux_reservoir());
    let mut bots = [HeuristicBot::new(), HeuristicBot::new(), HeuristicBot::new()];
    for _ in 0..200 {
        for (seat, bot) in bots.iter_mut().enumerate() {
            if let Some(a) = bot.next_action(&g, seat) {
                let _ = g.perform_action(a);
            }
            assert!(g.stack.len() <= 1, "shots stacked: {}", g.stack.len());
        }
        if g.is_game_over() {
            break;
        }
    }
    assert!(g.is_game_over(), "two 50-point shots end a three-seat game");
}
